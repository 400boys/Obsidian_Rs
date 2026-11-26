// Découverte et filtrage des fenêtres cibles Antigravity via l'API HTTP de DevTools

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdpTarget {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(rename = "type")]
    pub target_type: String,
    #[serde(default)]
    pub url: String,
    pub web_socket_debugger_url: Option<String>,
}

impl CdpTarget {
    /// Détermine intelligemment si une cible correspond à une fenêtre active éligible d'Antigravity (IDE ou extension VS Code).
    pub fn is_eligible(&self) -> bool {
        // En mode IDE autonome et en mode Extension VS Code, les cibles peuvent être de type "page", "iframe" ou "webview"
        if self.target_type != "page" && self.target_type != "iframe" && self.target_type != "webview" {
            return false;
        }

        let _ws = match &self.web_socket_debugger_url {
            Some(w) if !w.is_empty() => w,
            _ => return false,
        };

        let url_lower = self.url.to_lowercase();
        let title_lower = self.title.to_lowercase();

        // Exclusion des pages internes de DevTools
        if url_lower.starts_with("devtools://") || url_lower.starts_with("chrome-extension://") {
            return false;
        }

        // Cible prioritaire : Workbench Antigravity, hub local 127.0.0.1 ou Webview VS Code
        if url_lower.contains("127.0.0.1")
            || url_lower.contains("localhost")
            || url_lower.contains("workbench")
            || url_lower.contains("antigravity")
            || url_lower.contains("jetski")
            || title_lower.contains("antigravity")
            || title_lower.contains("gemini")
            || url_lower.starts_with("vscode-webview://")
            || url_lower.starts_with("vscode-file://")
            || url_lower.starts_with("file://")
        {
            return true;
        }

        // Par défaut, toute page locale avec titre non vide est admissible
        !self.title.is_empty()
    }
}

/// Récupère la liste de toutes les cibles auprès de l'instance locale d'Antigravity via HTTP.
pub async fn list_targets(port: u16) -> Result<Vec<CdpTarget>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| format!("Erreur de création du client HTTP: {}", e))?;

    let url = format!("http://127.0.0.1:{}/json/list", port);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Impossible de joindre Antigravity sur le port {}: {}", port, e))?;

    let targets: Vec<CdpTarget> = resp
        .json()
        .await
        .map_err(|e| format!("Réponse JSON invalide reçue de DevTools: {}", e))?;

    Ok(targets)
}

/// Récupère uniquement les cibles éligibles pour l'injection du Prisme d'Obsidienne.
pub async fn list_eligible_targets(port: u16) -> Result<Vec<CdpTarget>, String> {
    let all = list_targets(port).await?;
    let eligible: Vec<CdpTarget> = all.into_iter().filter(|t| t.is_eligible()).collect();
    Ok(eligible)
}
