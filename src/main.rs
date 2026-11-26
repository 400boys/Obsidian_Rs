// Obsidian_Rs v1.0.8 — Version 100% Rust du plugin Antigravity Obsidian Prism
// Suite Optique & Mouvement pour Google Antigravity
// Entièrement en français

use clap::{Args, Parser, Subcommand};
use colored::*;
use obsidian_rs::assets::load_style_css;
use obsidian_rs::cdp::{is_port_listening, list_eligible_targets, read_active_port};
use obsidian_rs::config::Config;
use obsidian_rs::injector;
use obsidian_rs::installer;
use obsidian_rs::watcher;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "obsidian_rs",
    author = "Edmond NOUMEGNI << Hacker Éthique >> <https://portfolio-nine-jade-22.vercel.app/>",
    version = "1.0.8",
    about = "Obsidian_Rs — Version 100% Rust de la Suite Optique & Mouvement pour Google Antigravity",
    long_about = "Obsidian_Rs est la version 100% Rust du plugin pour Google Antigravity.\n\
    Développé par Edmond NOUMEGNI << Hacker Éthique >> (https://portfolio-nine-jade-22.vercel.app/)\n\
    Il injecte des thèmes optiques cristallins, une fiole de télémétrie 512K en temps réel,\n\
    un curseur d'effort de raisonnement, des mascottes pixel interactives et des animations à ressorts fluides."
)]
struct Cli {
    #[command(subcommand)]
    commande: Option<Commandes>,
}

#[derive(Subcommand)]
enum Commandes {
    #[command(
        name = "injecter",
        alias = "inject",
        about = "Injecter le Prisme d'Obsidienne dans toutes les fenêtres Antigravity ouvertes"
    )]
    Injecter(InjecterArgs),

    #[command(
        name = "surveiller",
        aliases = &["watch", "daemon"],
        about = "Lancer le démon de surveillance continue (injection automatique et rechargement à chaud)"
    )]
    Surveiller {
        #[arg(short, long, help = "Démarrer le démon en arrière-plan de façon transparente")]
        arriere_plan: bool,
    },

    #[command(
        name = "arreter",
        aliases = &["stop", "kill"],
        about = "Arrêter le démon de surveillance en cours d'exécution"
    )]
    Arreter,

    #[command(
        name = "installer",
        aliases = &["install", "setup"],
        about = "Installer le plugin et configurer les raccourcis d'Antigravity avec le port de débogage"
    )]
    Installer,

    #[command(
        name = "desinstaller",
        aliases = &["uninstall", "remove"],
        about = "Désinstaller le plugin, restaurer les raccourcis d'origine et nettoyer les fenêtres"
    )]
    Desinstaller,

    #[command(
        name = "statut",
        alias = "status",
        about = "Afficher l'état d'Antigravity, du port CDP, des fenêtres et du démon"
    )]
    Statut,

    #[command(
        name = "config",
        alias = "configuration",
        about = "Afficher, modifier ou réinitialiser la configuration"
    )]
    Configuration(ConfigArgs),

    #[command(
        name = "relancer",
        aliases = &["relaunch", "restart"],
        about = "Relancer Antigravity avec le port de débogage activé (--remote-debugging-port=9222)"
    )]
    Relancer,
}

#[derive(Args)]
struct InjecterArgs {
    #[arg(short, long, help = "Chemin vers une feuille de style CSS personnalisée")]
    css: Option<PathBuf>,
}

#[derive(Args)]
struct ConfigArgs {
    #[command(subcommand)]
    action: Option<ConfigAction>,
}

#[derive(Subcommand)]
enum ConfigAction {
    #[command(name = "afficher", alias = "show", about = "Afficher la configuration actuelle")]
    Afficher,

    #[command(name = "modifier", alias = "set", about = "Modifier une clé de configuration")]
    Modifier {
        #[arg(short, long, help = "Clé de configuration (ex: palette, lang, mascot)")]
        cle: String,
        #[arg(short, long, help = "Nouvelle valeur")]
        valeur: String,
    },

    #[command(name = "reinitialiser", alias = "reset", about = "Réinitialiser aux valeurs françaises par défaut")]
    Reinitialiser,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.commande {
        Some(Commandes::Injecter(args)) => {
            println!("{}", "▶ Injection du Prisme d'Obsidienne dans Antigravity...".cyan().bold());
            let config = Config::load();
            let custom_css = args.css.as_ref().map(|p| load_style_css(p));

            match injector::inject_all(&config, custom_css.as_deref()).await {
                Ok(res) => {
                    println!("{}", "═".repeat(60).green());
                    println!("  {} Injection réussie dans {} fenêtre(s) :", "✔".green().bold(), res.injected_targets.len());
                    for title in res.injected_targets {
                        println!("    • {}", title.bold());
                    }
                    if !res.failed_targets.is_empty() {
                        println!("\n  {} Échecs sur {} fenêtre(s) :", "⚠".yellow(), res.failed_targets.len());
                        for (title, err) in res.failed_targets {
                            println!("    • {} : {}", title, err.red());
                        }
                    }
                    println!("{}", "═".repeat(60).green());
                }
                Err(e) => {
                    eprintln!("{} Échec de l'injection : {}", "✖".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commandes::Surveiller { arriere_plan }) => {
            if arriere_plan {
                let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("obsidian_rs.exe"));
                #[cfg(target_os = "windows")]
                {
                    let script = format!(
                        "Start-Process -FilePath '{}' -ArgumentList 'surveiller' -WindowStyle Hidden",
                        current_exe.to_string_lossy()
                    );
                    let res = Command::new("powershell")
                        .args(&["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script])
                        .spawn();
                    if let Err(e) = res {
                        eprintln!("{} Échec du démarrage en arrière-plan : {}", "✖".red(), e);
                        std::process::exit(1);
                    }
                    println!("{} Démon de surveillance démarré en arrière-plan.", "✔".green().bold());
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let res = Command::new(&current_exe).arg("surveiller").spawn();
                    if let Err(e) = res {
                        eprintln!("{} Échec du démarrage en arrière-plan : {}", "✖".red(), e);
                        std::process::exit(1);
                    }
                    println!("{} Démon de surveillance démarré en arrière-plan.", "✔".green().bold());
                }
            } else {
                if let Err(e) = watcher::run_daemon().await {
                    eprintln!("{} Erreur du démon : {}", "✖".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commandes::Arreter) => {
            let pid_path = std::path::Path::new(watcher::PID_FILE_NAME);
            if pid_path.exists() {
                if let Ok(content) = std::fs::read_to_string(pid_path) {
                    if let Ok(pid) = content.trim().parse::<u32>() {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = Command::new("taskkill").args(&["/PID", &pid.to_string(), "/F"]).output();
                        }
                        #[cfg(not(target_os = "windows"))]
                        {
                            let _ = Command::new("kill").arg(pid.to_string()).output();
                        }
                        let _ = std::fs::remove_file(pid_path);
                        println!("{} Démon de surveillance (PID {}) arrêté avec succès.", "✔".green().bold(), pid);
                        return;
                    }
                }
            }
            println!("{} Aucun démon de surveillance actif n'a été trouvé.", "ℹ".blue());
        }

        Some(Commandes::Installer) => {
            if let Err(e) = installer::run_installation().await {
                eprintln!("{} Erreur lors de l'installation : {}", "✖".red().bold(), e);
                std::process::exit(1);
            }
        }

        Some(Commandes::Desinstaller) => {
            if let Err(e) = installer::run_uninstallation().await {
                eprintln!("{} Erreur lors de la désinstallation : {}", "✖".red().bold(), e);
                std::process::exit(1);
            }
        }

        Some(Commandes::Statut) | None => {
            afficher_statut().await;
        }

        Some(Commandes::Configuration(args)) => match args.action {
            Some(ConfigAction::Afficher) | None => {
                let config = Config::load();
                config.print_summary();
            }
            Some(ConfigAction::Modifier { cle, valeur }) => {
                let mut config = Config::load();
                match config.set_property(&cle, &valeur) {
                    Ok(_) => {
                        if let Err(e) = config.save() {
                            eprintln!("{} Impossible d'enregistrer la configuration : {}", "✖".red(), e);
                            std::process::exit(1);
                        }
                        println!("{} Option '{}' mise à jour avec la valeur '{}'.", "✔".green(), cle.bold(), valeur.bold());
                        let _ = injector::broadcast_config_refresh(&config).await;
                    }
                    Err(e) => {
                        eprintln!("{} Clé invalide : {}", "✖".red(), e);
                    }
                }
            }
            Some(ConfigAction::Reinitialiser) => {
                let config = Config::default();
                if let Err(e) = config.save() {
                    eprintln!("{} Erreur lors de la réinitialisation : {}", "✖".red(), e);
                    std::process::exit(1);
                }
                println!("{} Configuration réinitialisée avec succès aux valeurs par défaut en français.", "✔".green());
                let _ = injector::broadcast_config_refresh(&config).await;
            }
        },

        Some(Commandes::Relancer) => {
            println!("{}", "▶ Redémarrage d'Antigravity avec le port de débogage...".cyan().bold());
            let detection = installer::detect::detect_antigravity();
            let exe_path = match detection.exe_path {
                Some(p) => p,
                None => {
                    eprintln!("{} Exécutable Antigravity introuvable.", "✖".red());
                    std::process::exit(1);
                }
            };

            // Fermeture des processus en cours
            for proc in detection.running_processes {
                #[cfg(target_os = "windows")]
                {
                    let _ = Command::new("taskkill")
                        .args(&["/PID", &proc.pid.to_string(), "/F"])
                        .output();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = Command::new("kill").arg(proc.pid.to_string()).output();
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(800)).await;

            // Lancement avec le drapeau de débogage
            #[cfg(target_os = "windows")]
            {
                let exe_str = exe_path.to_string_lossy().to_string();
                let res = Command::new("cmd")
                    .args(&["/c", "start", "", &exe_str, "--remote-debugging-port=9222"])
                    .env_remove("ELECTRON_RUN_AS_NODE")
                    .spawn();

                if let Err(e) = res {
                    eprintln!("{} Échec du lancement d'Antigravity : {}", "✖".red(), e);
                    std::process::exit(1);
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                let _ = Command::new(&exe_path)
                    .arg("--remote-debugging-port=9222")
                    .spawn();
            }

            println!("{} Antigravity a été relancé avec succès avec --remote-debugging-port=9222 !", "✔".green());
        }
    }
}

/// Affiche un bilan détaillé de l'état du système et d'Antigravity en français.
async fn afficher_statut() {
    println!("{}", "╔══════════════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║          DIAGNOSTIC ET STATUT DU PRISME D'OBSIDIENNE                 ║".cyan().bold());
    println!("{}", "║                  Suite 100% Rust pour Antigravity                    ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════════════╝".cyan());

    // 1. Processus Antigravity
    let detection = installer::detect::detect_antigravity();
    println!("\n{}", "1. Exécutable & Processus Antigravity".bold());
    if let Some(path) = &detection.exe_path {
        println!("  • Exécutable localisé : {}", path.to_string_lossy().green());
    } else {
        println!("  • Exécutable : {}", "Non localisé".red());
    }

    if detection.is_running {
        println!("  • État d'exécution : {}", "En cours d'exécution".green().bold());
        for p in &detection.running_processes {
            println!("    - PID {}: {}", p.pid, p.name);
        }
    } else {
        println!("  • État d'exécution : {}", "Inactif (Antigravity n'est pas lancé)".yellow());
    }

    // 2. Port de débogage CDP
    println!("\n{}", "2. Port de débogage DevTools (CDP)".bold());
    match read_active_port() {
        Ok(port_info) => {
            let listening = is_port_listening(port_info.port, 300);
            let state_str = if listening {
                "Actif et réactif".green().bold()
            } else {
                "Fichier présent mais port inaccessible".yellow()
            };
            println!("  • Port DevTools : {} ({})", port_info.port.to_string().cyan().bold(), state_str);
            println!("  • Fichier actif : {:?}", port_info.file_path);

            if listening {
                match list_eligible_targets(port_info.port).await {
                    Ok(targets) => {
                        println!("\n{}", "3. Fenêtres Antigravity Détectées".bold());
                        println!("  • Nombre de fenêtres éligibles : {}", targets.len().to_string().cyan().bold());
                        for (i, t) in targets.iter().enumerate() {
                            let title = if t.title.is_empty() { "Fenêtre principale" } else { &t.title };
                            println!("    [{}] {} (ID: {})", i + 1, title.green().bold(), &t.id[..8.min(t.id.len())]);
                            println!("        URL : {}", t.url.dimmed());
                        }
                    }
                    Err(e) => {
                        println!("  • Impossible de lister les cibles : {}", e.red());
                    }
                }
            }
        }
        Err(_) => {
            println!("  • Port DevTools : {}", "Aucun port actif détecté".yellow());
            println!("    (Lancez Antigravity avec 'prisme relancer' ou depuis le raccourci patché)");
        }
    }

    // 4. Démon de surveillance
    println!("\n{}", "4. Démon de Surveillance (Watch)".bold());
    let pid_path = std::path::Path::new(watcher::PID_FILE_NAME);
    if pid_path.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(pid_path) {
            println!("  • Démon actif : {} (PID: {})", "En cours".green().bold(), pid_str.trim());
        } else {
            println!("  • Démon : {}", "Inconnu".yellow());
        }
    } else {
        println!("  • Démon : {}", "Inactif (lancez 'prisme surveiller')".dimmed());
    }

    // 5. Configuration active
    println!("\n{}", "5. Configuration Active".bold());
    let config = Config::load();
    println!("  • Langue : {} | Palette : {} | Cinétique : {}", config.lang.cyan(), config.palette.cyan(), config.motion_level.cyan());
    println!("  • Fiole 512K : {} | Curseur de réflexion : {} | Mascotte : {}", config.telemetry_module.cyan(), config.effort_slider_module.cyan(), config.mascot.cyan());

    println!("\n{}", "─".repeat(60).dimmed());
    println!("Commandes rapides :");
    println!("  obsidian_rs injecter     -> Injecter immédiatement dans Antigravity");
    println!("  obsidian_rs surveiller   -> Démarrer le démon d'injection automatique");
    println!("  obsidian_rs installer    -> Configurer les raccourcis système");
    println!("  obsidian_rs relancer     -> Redémarrer Antigravity en mode débogage");
    println!("{}", "─".repeat(60).dimmed());

    println!("  {} Edmond NOUMEGNI << Hacker Éthique >>", "[+] Dev:".green().bold());
    println!("  {} https://portfolio-nine-jade-22.vercel.app/", "[*] Net:".green().bold());
    println!("{}", "─".repeat(60).dimmed());
}
