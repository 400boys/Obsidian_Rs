pub mod asar;
pub mod detect;
pub mod shortcuts;
pub mod vscode;

use crate::config::Config;
use crate::injector;
use colored::*;

#[derive(Debug)]
pub struct InstallStatus {
    pub antigravity_found: bool,
    pub exe_path: Option<String>,
    pub is_running: bool,
    pub shortcuts_patched: usize,
    pub already_patched: usize,
    pub injected_now: bool,
    pub injected_windows_count: usize,
}

/// Procède à l'installation complète du Prisme d'Obsidienne pour Antigravity.
pub async fn run_installation() -> Result<InstallStatus, String> {
    println!("{}", "═".repeat(65).cyan());
    println!("{}", "  INSTALLATION DU PRISME D'OBSIDIENNE (100% RUST)".cyan().bold());
    println!("  Suite Optique & Mouvement pour Google Antigravity");
    println!("{}", "═".repeat(65).cyan());

    // 1. Détection d'Antigravity
    let detection = detect::detect_antigravity();
    let antigravity_found = detection.exe_path.is_some();
    let exe_str = detection.exe_path.map(|p| p.to_string_lossy().to_string());

    if let Some(ref path) = exe_str {
        println!("  {} Antigravity détecté : {}", "✔".green(), path.bold());
    } else {
        println!("  {} Antigravity n'a pas été trouvé dans les chemins usuels.", "⚠".yellow());
    }

    if detection.is_running {
        println!("  {} Processus Antigravity en cours d'exécution ({} processus).", "ℹ".blue(), detection.running_processes.len());
    } else {
        println!("  {} Antigravity n'est pas lancé actuellement.", "ℹ".blue());
    }

    // 2. Patch des raccourcis
    println!("\n{}", "▶ Configuration des raccourcis...".bold());
    let patch_res = shortcuts::patch_antigravity_shortcuts()?;
    println!("  • Raccourcis analysés : {}", patch_res.scanned);
    println!("  • Raccourcis Antigravity trouvés : {}", patch_res.matched);
    println!("  • Raccourcis modifiés avec --remote-debugging-port : {}", patch_res.patched.len().to_string().green());
    println!("  • Déjà configurés : {}", patch_res.already_set.len());

    // 3. Initialisation et déploiement de la configuration et des fichiers du plugin
    println!("\n{}", "▶ Déploiement du plugin dans ~/.gemini/config/plugins/...".bold());
    let config = Config::load();
    let _ = config.save();

    if let Some(home) = dirs::home_dir() {
        let target_plugin_dir = home.join(".gemini").join("config").join("plugins").join("obsidian-prism");
        if let Ok(_) = std::fs::create_dir_all(&target_plugin_dir) {
            // Copie de plugin.json, config.json, style.css, et du binaire prisme.exe
            let files_to_copy = ["plugin.json", "package.json", "config.json", "style.css", "injecter.bat", "surveiller.bat", "desinstaller.bat"];
            for f in &files_to_copy {
                if std::path::Path::new(f).exists() {
                    let _ = std::fs::copy(f, target_plugin_dir.join(f));
                }
            }

            // Copie du dossier assets
            let target_assets = target_plugin_dir.join("assets");
            let _ = std::fs::create_dir_all(&target_assets);
            if std::path::Path::new("assets/logo.svg").exists() {
                let _ = std::fs::copy("assets/logo.svg", target_assets.join("logo.svg"));
            }

            // Copie du binaire compilé release (obsidian_rs.exe ou prisme.exe)
            let release_bin = std::path::Path::new("target/release/obsidian_rs.exe");
            if release_bin.exists() {
                let _ = std::fs::copy(release_bin, target_plugin_dir.join("obsidian_rs.exe"));
            } else {
                let fallback_bin = std::path::Path::new("target/release/prisme.exe");
                if fallback_bin.exists() {
                    let _ = std::fs::copy(fallback_bin, target_plugin_dir.join("obsidian_rs.exe"));
                }
            }

            println!("  {} Plugin déployé avec succès dans : {}", "✔".green(), target_plugin_dir.display().to_string().bold());
        }
    }
    println!("  {} Configuration sauvegardée (Langue: FR, Palette: Obsidienne, Cinétique: Ultra).", "✔".green());

    // 4. Intégration persistante dans app.asar (IDE autonome)
    println!("\n{}", "▶ Activation de l'intégration native persistante (app.asar - IDE)...".bold());
    let target_asar = detection.asar_path.or_else(|| {
        dirs::data_local_dir()
            .map(|d| d.join("Programs").join("antigravity").join("resources").join("app.asar"))
    });
    if let Some(asar_path) = target_asar {
        if asar_path.exists() {
            let inject_code = injector::compiler::build_inject_expression(&config, None);
            match asar::patch_asar_file(&asar_path, &inject_code) {
                Ok(_) => {
                    println!("  {} Intégration permanente activée dans app.asar (100% Rust) : {}", "✔".green(), asar_path.display().to_string().bold());
                }
                Err(e) => {
                    println!("  {} Avertissement lors du patch d'app.asar: {}", "⚠".yellow(), e);
                }
            }
        }
    }

    // 5. Intégration persistante dans l'extension VS Code / Cursor
    println!("\n{}", "▶ Recherche des extensions Antigravity pour VS Code / Cursor...".bold());
    let vscode_exts = vscode::find_antigravity_vscode_extensions();
    if vscode_exts.is_empty() {
        println!("  {} Aucune extension VS Code Antigravity trouvée.", "ℹ".blue());
    } else {
        let inject_code = injector::compiler::build_inject_expression(&config, None);
        for ext in &vscode_exts {
            match vscode::patch_vscode_extension(ext, &inject_code) {
                Ok(_) => {
                    println!("  {} Intégration permanente activée dans l'extension VS Code : {}", "✔".green(), ext.name.bold());
                }
                Err(e) => {
                    println!("  {} Avertissement lors du patch de l'extension {}: {}", "⚠".yellow(), ext.name, e);
                }
            }
        }
    }

    // 6. Enregistrement du démon de surveillance au démarrage système (Autorun Windows)
    #[cfg(target_os = "windows")]
    {
        println!("\n{}", "▶ Configuration de la surveillance automatique au démarrage de Windows...".bold());
        let current_exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("obsidian_rs.exe"));
        let autorun_cmd = format!("\"{}\" surveiller", current_exe.to_string_lossy());
        let reg_res = std::process::Command::new("reg")
            .args(&[
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "Obsidian_Rs",
                "/t",
                "REG_SZ",
                "/d",
                &autorun_cmd,
                "/f",
            ])
            .output();
        if let Ok(out) = reg_res {
            if out.status.success() {
                println!("  {} Démarrage automatique configuré (surveillance transparente pour IDE et VS Code).", "✔".green());
            }
        }
    }

    // 7. Tentative d'injection immédiate si Antigravity ou VS Code tourne déjà avec le port de débogage
    let mut injected_now = false;
    let mut injected_count = 0;

    println!("\n{}", "▶ Recherche de fenêtres actives pour injection (IDE & VS Code)...".bold());
    match injector::inject_all(&config, None).await {
        Ok(inj_res) => {
            injected_now = true;
            injected_count = inj_res.injected_targets.len();
            println!("  {} Injection réussie dans {} fenêtre(s) :", "✔".green(), injected_count);
            for t in inj_res.injected_targets {
                println!("    • {}", t.bold());
            }
        }
        Err(e) => {
            println!("  {} Port de débogage non joignable immédiatement ({})", "ℹ".blue(), e);
            if detection.is_running {
                println!("  {} Veuillez redémarrer Antigravity depuis votre raccourci pour activer le port de débogage.", "💡".yellow());
            } else {
                println!("  {} Lancez simplement Antigravity depuis votre raccourci pour profiter du Prisme !", "💡".green());
            }
        }
    }

    println!("\n{}", "═".repeat(65).cyan());
    println!("{}", "  INSTALLATION TERMINÉE AVEC SUCCÈS !".green().bold());
    println!("  Pour activer la surveillance automatique des fenêtres :");
    println!("    cargo run -- surveiller");
    println!("{}", "═".repeat(65).cyan());

    Ok(InstallStatus {
        antigravity_found,
        exe_path: exe_str,
        is_running: detection.is_running,
        shortcuts_patched: patch_res.patched.len(),
        already_patched: patch_res.already_set.len(),
        injected_now,
        injected_windows_count: injected_count,
    })
}

#[derive(Debug)]
pub struct UninstallStatus {
    pub cleaned_windows: usize,
    pub shortcuts_restored: usize,
}

/// Procède à la désinstallation complète et propre du Prisme d'Obsidienne.
pub async fn run_uninstallation() -> Result<UninstallStatus, String> {
    println!("{}", "═".repeat(65).yellow());
    println!("{}", "  DÉSINSTALLATION DU PRISME D'OBSIDIENNE".yellow().bold());
    println!("{}", "═".repeat(65).yellow());

    // 1. Nettoyage de l'interface dans les fenêtres ouvertes
    println!("\n{}", "▶ Nettoyage des fenêtres Antigravity ouvertes...".bold());
    let mut cleaned_windows = 0;
    match injector::remove_all().await {
        Ok(res) => {
            cleaned_windows = res.cleaned_targets.len();
            if cleaned_windows > 0 {
                println!("  {} Style et scripts retirés de {} fenêtre(s).", "✔".green(), cleaned_windows);
            } else {
                println!("  {} Aucune fenêtre active connectée.", "ℹ".blue());
            }
        }
        Err(_) => {
            println!("  {} Aucune session CDP active.", "ℹ".blue());
        }
    }

    // 2. Restauration de app.asar (100% Rust)
    println!("\n{}", "▶ Restauration de app.asar (100% Rust)...".bold());
    let detection = detect::detect_antigravity();
    let target_asar = detection.asar_path.or_else(|| {
        dirs::data_local_dir()
            .map(|d| d.join("Programs").join("antigravity").join("resources").join("app.asar"))
    });
    if let Some(asar_path) = target_asar {
        if asar_path.exists() {
            let _ = asar::restore_asar_file(&asar_path);
            println!("  {} Fichier app.asar restauré dans son état natif (100% Rust).", "✔".green());
        }
    }

    // 3. Restauration des extensions VS Code
    println!("\n{}", "▶ Restauration des extensions VS Code / Cursor...".bold());
    let vscode_exts = vscode::find_antigravity_vscode_extensions();
    let mut restored_exts = 0;
    for ext in &vscode_exts {
        if vscode::restore_vscode_extension(ext).unwrap_or(false) {
            restored_exts += 1;
        }
    }
    if restored_exts > 0 {
        println!("  {} {} extension(s) VS Code restaurée(s).", "✔".green(), restored_exts);
    }

    // 4. Restauration des raccourcis
    println!("\n{}", "▶ Restauration des raccourcis d'origine...".bold());
    let restored = shortcuts::restore_antigravity_shortcuts()?;
    println!("  {} {} raccourci(s) restauré(s).", "✔".green(), restored);

    // 5. Suppression de l'autorun Windows
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("reg")
            .args(&[
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "Obsidian_Rs",
                "/f",
            ])
            .output();
    }

    println!("\n{}", "═".repeat(65).green());
    println!("{}", "  DÉSINSTALLATION TERMINÉE !".green().bold());
    println!("  Antigravity a été restauré dans son état natif.");
    println!("{}", "═".repeat(65).green());

    Ok(UninstallStatus {
        cleaned_windows,
        shortcuts_restored: restored,
    })
}
