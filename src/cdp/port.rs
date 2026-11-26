// Détection et résolution adaptative du port CDP (Chrome DevTools Protocol) pour Antigravity
// S'adapte automatiquement à l'IDE autonome ET à l'extension VS Code / Cursor / VSCodium.

use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

/// Renvoie les chemins d'accès potentiels aux fichiers DevToolsActivePort (IDE Antigravity, VS Code, Cursor).
pub fn get_candidate_port_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        let mut base_dirs = Vec::new();
        if let Ok(app_data) = std::env::var("APPDATA") {
            base_dirs.push(PathBuf::from(app_data));
        }
        if let Some(home) = dirs::home_dir() {
            base_dirs.push(home.join("AppData").join("Roaming"));
        }

        for base in base_dirs {
            // IDE Antigravity
            paths.push(base.join("Antigravity").join("DevToolsActivePort"));
            paths.push(base.join("antigravity").join("DevToolsActivePort"));
            // Visual Studio Code & Cursor & VSCodium
            paths.push(base.join("Code").join("DevToolsActivePort"));
            paths.push(base.join("Cursor").join("DevToolsActivePort"));
            paths.push(base.join("VSCodium").join("DevToolsActivePort"));
            paths.push(base.join("Code - Insiders").join("DevToolsActivePort"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let app_sup = home.join("Library").join("Application Support");
            paths.push(app_sup.join("Antigravity").join("DevToolsActivePort"));
            paths.push(app_sup.join("Code").join("DevToolsActivePort"));
            paths.push(app_sup.join("Cursor").join("DevToolsActivePort"));
            paths.push(app_sup.join("VSCodium").join("DevToolsActivePort"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        let mut base_dirs = Vec::new();
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            base_dirs.push(PathBuf::from(xdg));
        }
        if let Some(home) = dirs::home_dir() {
            base_dirs.push(home.join(".config"));
        }

        for base in base_dirs {
            paths.push(base.join("Antigravity").join("DevToolsActivePort"));
            paths.push(base.join("Code").join("DevToolsActivePort"));
            paths.push(base.join("Cursor").join("DevToolsActivePort"));
            paths.push(base.join("VSCodium").join("DevToolsActivePort"));
        }
    }

    paths
}

pub fn get_devtools_active_port_path() -> Option<PathBuf> {
    for p in get_candidate_port_paths() {
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Information extraite du fichier DevToolsActivePort ou de la découverte dynamique.
#[derive(Debug, Clone)]
pub struct DevToolsPortInfo {
    pub port: u16,
    pub browser_path: String,
    pub file_path: PathBuf,
}

/// Lit et analyse le port DevTools avec résolution intelligente et repli automatique.
pub fn read_active_port() -> Result<DevToolsPortInfo, String> {
    // 1. Priorité aux variables d'environnement (si l'utilisateur a configuré un port personnalisé)
    for env_var in ["AG_CDP_PORT", "CDP_PORT", "ANTIGRAVITY_PORT", "VSCODE_PORT"] {
        if let Ok(val) = std::env::var(env_var) {
            if let Ok(p) = val.trim().parse::<u16>() {
                if is_port_listening(p, 150) {
                    return Ok(DevToolsPortInfo {
                        port: p,
                        browser_path: String::new(),
                        file_path: PathBuf::from(format!("env:{}", env_var)),
                    });
                }
            }
        }
    }

    // 2. Découverte via les fichiers DevToolsActivePort actifs (Antigravity IDE ou VS Code)
    for port_file in get_candidate_port_paths() {
        if port_file.exists() {
            if let Ok(content) = fs::read_to_string(&port_file) {
                let lines: Vec<&str> = content.lines().collect();
                if !lines.is_empty() {
                    if let Ok(port) = lines[0].trim().parse::<u16>() {
                        if is_port_listening(port, 150) {
                            let browser_path = if lines.len() > 1 {
                                lines[1].trim().to_string()
                            } else {
                                String::new()
                            };
                            return Ok(DevToolsPortInfo {
                                port,
                                browser_path,
                                file_path: port_file,
                            });
                        }
                    }
                }
            }
        }
    }

    // 3. Détection par scan rapide sur les ports standards (9222, 9229, 9333) si l'un d'eux tourne
    for candidate_port in [9222, 9229, 9333] {
        if is_port_listening(candidate_port, 100) {
            return Ok(DevToolsPortInfo {
                port: candidate_port,
                browser_path: String::new(),
                file_path: PathBuf::from(format!("tcp://127.0.0.1:{}", candidate_port)),
            });
        }
    }

    Err("Aucun port de débogage DevTools actif n'a pu être localisé (ni dans l'IDE Antigravity, ni dans VS Code). Lancez votre application depuis un raccourci ou avec 'obsidian_rs relancer'.".to_string())
}

/// Vérifie rapidement si le port TCP spécifié est actif et répond.
pub fn is_port_listening(port: u16, timeout_ms: u64) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    if let Ok(stream) = TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        Duration::from_millis(timeout_ms),
    ) {
        drop(stream);
        true
    } else {
        false
    }
}
