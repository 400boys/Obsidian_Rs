// Détection universelle et intelligente de l'installation et des processus d'Antigravity
// S'adapte automatiquement à n'importe quel PC (Windows, macOS, Linux, chemins personnalisés, registres et disques secondaires).

use std::path::{Path, PathBuf};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

#[derive(Debug, Clone)]
pub struct AntigravityProcessInfo {
    pub pid: u32,
    pub name: String,
    pub exe_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct AntigravityInstallation {
    pub exe_path: Option<PathBuf>,
    pub asar_path: Option<PathBuf>,
    pub is_running: bool,
    pub running_processes: Vec<AntigravityProcessInfo>,
}

/// Détecte l'emplacement de l'exécutable Antigravity sur n'importe quelle machine.
pub fn find_antigravity_executable() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // 1. Chemins utilisateurs (LocalAppData & AppData) avec gestion des variations de casse
        if let Some(local_app) = dirs::data_local_dir() {
            let candidates = [
                local_app.join("Programs").join("antigravity").join("Antigravity.exe"),
                local_app.join("Programs").join("Antigravity").join("Antigravity.exe"),
                local_app.join("antigravity").join("Antigravity.exe"),
                local_app.join("Antigravity").join("Antigravity.exe"),
            ];
            for c in &candidates {
                if c.exists() {
                    return Some(c.clone());
                }
            }
        }

        if let Ok(app_data) = std::env::var("APPDATA") {
            let candidates = [
                PathBuf::from(&app_data).join("Programs").join("antigravity").join("Antigravity.exe"),
                PathBuf::from(&app_data).join("Programs").join("Antigravity").join("Antigravity.exe"),
                PathBuf::from(&app_data).join("antigravity").join("Antigravity.exe"),
            ];
            for c in &candidates {
                if c.exists() {
                    return Some(c.clone());
                }
            }
        }

        // 2. Répertoires d'installation système (ProgramFiles 64-bit et 32-bit)
        let prog_env_vars = ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"];
        for var in &prog_env_vars {
            if let Ok(dir) = std::env::var(var) {
                let candidates = [
                    PathBuf::from(&dir).join("Antigravity").join("Antigravity.exe"),
                    PathBuf::from(&dir).join("antigravity").join("Antigravity.exe"),
                ];
                for c in &candidates {
                    if c.exists() {
                        return Some(c.clone());
                    }
                }
            }
        }

        // 3. Recherche sur les disques secondaires courants (D:\, E:\)
        for drive in ["C", "D", "E", "F"] {
            let candidates = [
                format!(r"{}:\Program Files\Antigravity\Antigravity.exe", drive),
                format!(r"{}:\Program Files\antigravity\Antigravity.exe", drive),
                format!(r"{}:\Antigravity\Antigravity.exe", drive),
            ];
            for c in &candidates {
                let p = PathBuf::from(c);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        // 4. Recherche dans le PATH système
        if let Ok(output) = std::process::Command::new("where")
            .arg("Antigravity.exe")
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    let p = PathBuf::from(first_line.trim());
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let candidates = [
            PathBuf::from("/Applications/Antigravity.app/Contents/MacOS/Antigravity"),
            PathBuf::from("/Applications/Google Antigravity.app/Contents/MacOS/Antigravity"),
        ];
        for c in &candidates {
            if c.exists() {
                return Some(c.clone());
            }
        }
        if let Some(home) = dirs::home_dir() {
            let user_app = home.join("Applications").join("Antigravity.app").join("Contents").join("MacOS").join("Antigravity");
            if user_app.exists() {
                return Some(user_app);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let candidates = [
            "/usr/bin/antigravity",
            "/usr/local/bin/antigravity",
            "/opt/antigravity/antigravity",
            "/opt/Antigravity/antigravity",
            "/snap/bin/antigravity",
            "/var/lib/flatpak/exports/bin/com.google.antigravity",
        ];
        for c in &candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                return Some(p);
            }
        }
        if let Some(home) = dirs::home_dir() {
            let user_candidates = [
                home.join(".local").join("bin").join("antigravity"),
                home.join(".local").join("share").join("flatpak").join("exports").join("bin").join("com.google.antigravity"),
            ];
            for uc in &user_candidates {
                if uc.exists() {
                    return Some(uc.clone());
                }
            }
        }
    }

    None
}

/// Résout l'emplacement exact du fichier app.asar associé à Antigravity sur n'importe quel PC.
pub fn find_antigravity_asar(exe_path: Option<&Path>) -> Option<PathBuf> {
    // 1. Déduction directe et fiable depuis l'exécutable détecté
    if let Some(exe) = exe_path {
        #[cfg(target_os = "macos")]
        {
            // Sur macOS : Antigravity.app/Contents/Resources/app.asar
            if let Some(parent) = exe.parent().and_then(|p| p.parent()) {
                let candidate = parent.join("Resources").join("app.asar");
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Sur Windows et Linux : <exe_dir>/resources/app.asar
            if let Some(parent) = exe.parent() {
                let candidate = parent.join("resources").join("app.asar");
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    // 2. Chemins de repli universels
    #[cfg(target_os = "windows")]
    {
        if let Some(local_app) = dirs::data_local_dir() {
            let candidates = [
                local_app.join("Programs").join("antigravity").join("resources").join("app.asar"),
                local_app.join("Programs").join("Antigravity").join("resources").join("app.asar"),
                local_app.join("antigravity").join("resources").join("app.asar"),
                local_app.join("Antigravity").join("resources").join("app.asar"),
            ];
            for c in &candidates {
                if c.exists() {
                    return Some(c.clone());
                }
            }
        }
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            let candidate = PathBuf::from(prog_files).join("Antigravity").join("resources").join("app.asar");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let candidate = PathBuf::from("/Applications/Antigravity.app/Contents/Resources/app.asar");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let candidates = [
            PathBuf::from("/usr/lib/antigravity/resources/app.asar"),
            PathBuf::from("/opt/antigravity/resources/app.asar"),
        ];
        for c in &candidates {
            if c.exists() {
                return Some(c.clone());
            }
        }
    }

    None
}

/// Analyse le système de manière adaptative pour détecter Antigravity en cours d'exécution ou installé.
pub fn detect_antigravity() -> AntigravityInstallation {
    let mut s = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    s.refresh_processes(ProcessesToUpdate::All, true);

    let mut running = Vec::new();
    let mut exe_from_proc: Option<PathBuf> = None;

    for (pid, process) in s.processes() {
        let name = process.name().to_string_lossy().to_string();
        let name_lower = name.to_lowercase();
        if name_lower == "antigravity.exe" || name_lower == "antigravity" {
            let path = process.exe().map(|p| p.to_path_buf());
            if exe_from_proc.is_none() && path.is_some() {
                exe_from_proc = path.clone();
            }
            running.push(AntigravityProcessInfo {
                pid: pid.as_u32(),
                name,
                exe_path: path,
            });
        }
    }

    let exe_path = exe_from_proc.or_else(find_antigravity_executable);
    let asar_path = find_antigravity_asar(exe_path.as_deref());
    let is_running = !running.is_empty();

    AntigravityInstallation {
        exe_path,
        asar_path,
        is_running,
        running_processes: running,
    }
}
