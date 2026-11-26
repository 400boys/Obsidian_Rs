// Client WebSocket asynchrone haute performance pour Chrome DevTools Protocol (CDP)
// Permet d'évaluer du code, d'ajouter des scripts persistants et de lier des RPC.

use crate::cdp::messages::{CdpRequest, CdpResponse};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct CdpClient {
    ws_url: String,
    stream: Arc<Mutex<WsStream>>,
    seq: AtomicU64,
}

impl CdpClient {
    /// Établit une connexion WebSocket avec la cible DevTools spécifiée.
    pub async fn connect(ws_url: &str) -> Result<Self, String> {
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| format!("Échec de connexion WebSocket à '{}': {}", ws_url, e))?;

        Ok(Self {
            ws_url: ws_url.to_string(),
            stream: Arc::new(Mutex::new(ws_stream)),
            seq: AtomicU64::new(1),
        })
    }

    /// Envoie une commande CDP et attend la réponse correspondante.
    pub async fn send_command(
        &self,
        method: &str,
        params: Option<Value>,
        timeout_duration: Duration,
    ) -> Result<Value, String> {
        let id = self.seq.fetch_add(1, Ordering::SeqCst);
        let req = CdpRequest::new(id, method, params);
        let req_json = serde_json::to_string(&req)
            .map_err(|e| format!("Erreur de sérialisation CDP: {}", e))?;

        let mut lock = self.stream.lock().await;
        lock.send(tokio_tungstenite::tungstenite::Message::Text(req_json.into()))
            .await
            .map_err(|e| format!("Échec d'envoi du message WebSocket: {}", e))?;

        // Attente de la réponse avec l'ID correspondant
        let response_fut = async {
            loop {
                let msg = match lock.next().await {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => return Err(format!("Erreur lors de la lecture du flux WebSocket: {}", e)),
                    None => return Err("Flux WebSocket fermé inopinément".to_string()),
                };

                if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
                    let resp: CdpResponse = match serde_json::from_str(&txt) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };

                    if resp.id == Some(id) {
                        if let Some(err) = resp.error {
                            return Err(format!("Erreur CDP [{}]: {}", err.code, err.message));
                        }
                        return Ok(resp.result.unwrap_or(Value::Null));
                    }
                }
            }
        };

        timeout(timeout_duration, response_fut)
            .await
            .map_err(|_| format!("Délai dépassé pour la commande CDP '{}' (ID {})", method, id))?
    }

    /// Active le domaine Page dans la cible.
    pub async fn page_enable(&self) -> Result<(), String> {
        self.send_command("Page.enable", None, Duration::from_secs(5))
            .await?;
        Ok(())
    }

    /// Active le domaine Runtime dans la cible.
    pub async fn runtime_enable(&self) -> Result<(), String> {
        self.send_command("Runtime.enable", None, Duration::from_secs(5))
            .await?;
        Ok(())
    }

    /// Ajoute un script qui s'exécutera automatiquement à chaque création de document (survit aux rechargements).
    pub async fn add_script_to_evaluate_on_new_document(&self, source: &str) -> Result<String, String> {
        let params = json!({ "source": source });
        let result = self
            .send_command("Page.addScriptToEvaluateOnNewDocument", Some(params), Duration::from_secs(10))
            .await?;

        let script_id = result["identifier"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(script_id)
    }

    /// Évalue une expression JavaScript immédiatement dans le contexte de la page.
    pub async fn evaluate(&self, expression: &str) -> Result<Value, String> {
        let params = json!({
            "expression": expression,
            "awaitPromise": true,
            "returnByValue": true,
            "userGesture": true
        });

        let result = self
            .send_command("Runtime.evaluate", Some(params), Duration::from_secs(15))
            .await?;

        Ok(result)
    }

    /// Ajoute une liaison bidirectionnelle (Binding) accessible depuis JavaScript.
    pub async fn add_binding(&self, name: &str) -> Result<(), String> {
        let params = json!({ "name": name });
        // Si la liaison existe déjà, ignore l'erreur
        let _ = self.send_command("Runtime.addBinding", Some(params), Duration::from_secs(5)).await;
        Ok(())
    }

    /// Récupère l'URL WebSocket de cette session.
    pub fn ws_url(&self) -> &str {
        &self.ws_url
    }
}
