// Démon de surveillance, d'injection automatique et de rechargement à chaud (Hot-Reload)

use crate::assets::load_style_css;
use crate::cdp::{is_port_listening, list_eligible_targets, read_active_port, CdpClient};
use crate::config::Config;
use crate::injector::{self, compiler};
use crate::watcher::lock::{PidLock, PID_FILE_NAME};
use colored::*;
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

pub async fn run_daemon() -> Result<(), String> {
    let _lock = PidLock::acquire(PID_FILE_NAME)?;

    println!("{}", "╔══════════════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║      DÉMON DE SURVEILLANCE DU PRISME D'OBSIDIENNE (100% RUST)        ║".cyan().bold());
    println!("{}", "║      Injection automatique & Rechargement à chaud en temps réel      ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════════════╝".cyan());
    println!("  {} Démon actif (PID: {}). Appuyez sur Ctrl+C pour quitter.\n", "●".green(), std::process::id());

    let mut injected_targets: HashSet<String> = HashSet::new();

    // Initialisation des empreintes
    let css_file = "style.css";
    let config_file = "config.json";

    let mut current_config = Config::load();
    let mut last_config_hash = compiler::compute_sha256(&current_config.to_compact_json());
    let current_css = load_style_css(css_file);
    let mut last_css_hash = compiler::compute_sha256(&current_css);

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\n{}", " Arrêt du démon demandé par l'utilisateur...".yellow().bold());
                break;
            }
            _ = sleep(Duration::from_millis(1500)) => {
                // 1. Vérification du rechargement à chaud de style.css
                if Path::new(css_file).exists() {
                    let disk_css = load_style_css(css_file);
                    let new_hash = compiler::compute_sha256(&disk_css);
                    if new_hash != last_css_hash {
                        last_css_hash = new_hash;
                        println!("  {} Modification détectée dans {} -> Diffusion à chaud du CSS...", "⚡".yellow(), css_file.bold());
                        match injector::broadcast_css_refresh(&disk_css).await {
                            Ok(n) => println!("    {} Feuille de style actualisée sur {} fenêtre(s).", "✔".green(), n),
                            Err(e) => eprintln!("    {} Échec de l'actualisation CSS: {}", "✖".red(), e),
                        }
                    }
                }

                // 2. Vérification du rechargement à chaud de config.json
                if Path::new(config_file).exists() {
                    if let Ok(new_cfg) = Config::load_from_path(config_file) {
                        let new_hash = compiler::compute_sha256(&new_cfg.to_compact_json());
                        if new_hash != last_config_hash {
                            last_config_hash = new_hash;
                            current_config = new_cfg;
                            println!("  {} Modification détectée dans {} -> Diffusion à chaud de la configuration...", "⚡".yellow(), config_file.bold());
                            match injector::broadcast_config_refresh(&current_config).await {
                                Ok(n) => println!("    {} Configuration actualisée sur {} fenêtre(s).", "✔".green(), n),
                                Err(e) => eprintln!("    {} Échec de l'actualisation config: {}", "✖".red(), e),
                            }
                        }
                    }
                }

                // 3. Détection des fenêtres Antigravity ouvertes
                let port_info = match read_active_port() {
                    Ok(p) => p,
                    Err(_) => {
                        // Port pas encore actif, reset la liste des cibles injectées
                        if !injected_targets.is_empty() {
                            println!("  {} Antigravity a été fermé. En attente de réouverture...", "ℹ".blue());
                            injected_targets.clear();
                        }
                        continue;
                    }
                };

                if !is_port_listening(port_info.port, 300) {
                    continue;
                }

                let targets = match list_eligible_targets(port_info.port).await {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                // Nettoyage des cibles qui n'existent plus
                let active_target_ids: HashSet<String> = targets.iter().map(|t| t.id.clone()).collect();
                injected_targets.retain(|id| active_target_ids.contains(id));

                // Injection dans les nouvelles fenêtres
                for target in &targets {
                    if !injected_targets.contains(&target.id) {
                        if let Some(ws_url) = &target.web_socket_debugger_url {
                            let title = if target.title.is_empty() {
                                format!("Fenêtre [{}]", &target.id[..8.min(target.id.len())])
                            } else {
                                target.title.clone()
                            };

                            println!("  {} Nouvelle fenêtre Antigravity détectée : {}", "✦".cyan(), title.bold());
                            let css_content = load_style_css(css_file);
                            let expr = compiler::build_inject_expression(&current_config, Some(&css_content));

                            match CdpClient::connect(ws_url).await {
                                Ok(client) => {
                                    let _ = client.page_enable().await;
                                    let _ = client.runtime_enable().await;
                                    let _ = client.add_binding(compiler::SETTING_BINDING).await;
                                    let _ = client.add_script_to_evaluate_on_new_document(&expr).await;

                                    match client.evaluate(&expr).await {
                                        Ok(_) => {
                                            println!("    {} Prisme d'Obsidienne injecté avec succès !", "✔".green());
                                            injected_targets.insert(target.id.clone());
                                        }
                                        Err(e) => {
                                            eprintln!("    {} Erreur lors de l'évaluation : {}", "✖".red(), e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("    {} Échec de connexion WebSocket : {}", "✖".red(), e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("  {} Démon arrêté proprement.", "✔".green());
    Ok(())
}
