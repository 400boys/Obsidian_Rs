pub mod compiler;

use crate::cdp::{list_eligible_targets, read_active_port, CdpClient};
use crate::config::Config;

#[derive(Debug, Default)]
pub struct InjectResult {
    pub total_targets: usize,
    pub injected_targets: Vec<String>,
    pub failed_targets: Vec<(String, String)>,
}

/// Injecte le Prisme d'Obsidienne dans toutes les fenêtres Antigravity ouvertes.
pub async fn inject_all(config: &Config, custom_css: Option<&str>) -> Result<InjectResult, String> {
    let port_info = read_active_port()?;
    let targets = list_eligible_targets(port_info.port).await?;

    if targets.is_empty() {
        return Err(format!(
            "Aucune fenêtre Antigravity éligible trouvée sur le port {}.",
            port_info.port
        ));
    }

    let expr = compiler::build_inject_expression(config, custom_css);
    let mut result = InjectResult {
        total_targets: targets.len(),
        ..Default::default()
    };

    for target in targets {
        let ws_url = match &target.web_socket_debugger_url {
            Some(u) => u.clone(),
            None => continue,
        };

        let title = if target.title.is_empty() {
            format!("Fenêtre [{}]", &target.id[..8.min(target.id.len())])
        } else {
            target.title.clone()
        };

        match CdpClient::connect(&ws_url).await {
            Ok(client) => {
                let _ = client.page_enable().await;
                let _ = client.runtime_enable().await;
                let _ = client.add_binding(compiler::SETTING_BINDING).await;

                // Script permanent pour les rechargements de page (F5 / Ctrl+R)
                let _ = client.add_script_to_evaluate_on_new_document(&expr).await;

                // Injection immédiate dans la page active
                match client.evaluate(&expr).await {
                    Ok(val) => {
                        let ok = val["result"]["value"]["ok"].as_bool().unwrap_or(true);
                        if ok {
                            result.injected_targets.push(title);
                        } else {
                            let err_msg = val["result"]["value"]["erreur"]
                                .as_str()
                                .unwrap_or("Erreur d'exécution")
                                .to_string();
                            result.failed_targets.push((title, err_msg));
                        }
                    }
                    Err(e) => {
                        result.failed_targets.push((title, e));
                    }
                }
            }
            Err(e) => {
                result.failed_targets.push((title, e));
            }
        }
    }

    Ok(result)
}

#[derive(Debug, Default)]
pub struct RemoveResult {
    pub total_targets: usize,
    pub cleaned_targets: Vec<String>,
    pub failed_targets: Vec<(String, String)>,
}

/// Supprime le Prisme d'Obsidienne de toutes les fenêtres Antigravity ouvertes.
pub async fn remove_all() -> Result<RemoveResult, String> {
    let port_info = read_active_port()?;
    let targets = list_eligible_targets(port_info.port).await?;

    if targets.is_empty() {
        return Ok(RemoveResult::default());
    }

    let remove_expr = compiler::build_remove_expression();
    let mut result = RemoveResult {
        total_targets: targets.len(),
        ..Default::default()
    };

    for target in targets {
        let ws_url = match &target.web_socket_debugger_url {
            Some(u) => u.clone(),
            None => continue,
        };

        let title = if target.title.is_empty() {
            format!("Fenêtre [{}]", &target.id[..8.min(target.id.len())])
        } else {
            target.title.clone()
        };

        match CdpClient::connect(&ws_url).await {
            Ok(client) => {
                let _ = client.runtime_enable().await;
                match client.evaluate(remove_expr).await {
                    Ok(_) => {
                        result.cleaned_targets.push(title);
                    }
                    Err(e) => {
                        result.failed_targets.push((title, e));
                    }
                }
            }
            Err(e) => {
                result.failed_targets.push((title, e));
            }
        }
    }

    Ok(result)
}

/// Diffuse à chaud la nouvelle configuration à toutes les fenêtres Antigravity ouvertes.
pub async fn broadcast_config_refresh(config: &Config) -> Result<usize, String> {
    let port_info = match read_active_port() {
        Ok(p) => p,
        Err(_) => return Ok(0),
    };
    let targets = list_eligible_targets(port_info.port).await.unwrap_or_default();
    let expr = compiler::build_config_refresh_expression(config);
    let mut updated = 0;

    for target in targets {
        if let Some(ws_url) = &target.web_socket_debugger_url {
            if let Ok(client) = CdpClient::connect(ws_url).await {
                if client.evaluate(&expr).await.is_ok() {
                    updated += 1;
                }
            }
        }
    }
    Ok(updated)
}

/// Diffuse à chaud la nouvelle feuille de style CSS à toutes les fenêtres Antigravity ouvertes.
pub async fn broadcast_css_refresh(new_css: &str) -> Result<usize, String> {
    let port_info = match read_active_port() {
        Ok(p) => p,
        Err(_) => return Ok(0),
    };
    let targets = list_eligible_targets(port_info.port).await.unwrap_or_default();
    let expr = compiler::build_css_refresh_expression(new_css);
    let mut updated = 0;

    for target in targets {
        if let Some(ws_url) = &target.web_socket_debugger_url {
            if let Ok(client) = CdpClient::connect(ws_url).await {
                if client.evaluate(&expr).await.is_ok() {
                    updated += 1;
                }
            }
        }
    }
    Ok(updated)
}
