//! Lighthouse client implementation

use crate::{epoch::Epoch, error::LighthouseError};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// Configuration for the Lighthouse client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighthouseConfig {
    /// HTTP endpoint for REST API
    pub http_url: String,
    /// WebSocket endpoint for epoch subscriptions
    pub ws_url: String,
    /// API key (optional, for authenticated endpoints)
    pub api_key: Option<String>,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
}

impl Default for LighthouseConfig {
    fn default() -> Self {
        Self {
            http_url: "https://lighthouse.fairseq.io".to_string(),
            ws_url: "wss://lighthouse.fairseq.io/ws".to_string(),
            api_key: None,
            timeout_secs: 30,
        }
    }
}

/// Client for interacting with the Lighthouse Network
pub struct LighthouseClient {
    config: LighthouseConfig,
    http_client: reqwest::Client,
    current_epoch: Arc<RwLock<Option<Epoch>>>,
    epoch_sender: broadcast::Sender<Epoch>,
}

impl LighthouseClient {
    /// Create a new Lighthouse client
    pub fn new(config: LighthouseConfig) -> Self {
        let (epoch_sender, _) = broadcast::channel(100);

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            current_epoch: Arc::new(RwLock::new(None)),
            epoch_sender,
        }
    }

    /// Get the current epoch from the Lighthouse
    pub async fn get_current_epoch(&self) -> Result<Epoch, LighthouseError> {
        let url = format!("{}/v1/epoch/current", self.config.http_url);

        let mut request = self.http_client.get(&url);
        if let Some(api_key) = &self.config.api_key {
            request = request.header("x-api-key", api_key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| LighthouseError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(LighthouseError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let epoch: Epoch = response
            .json()
            .await
            .map_err(|e| LighthouseError::Parse(e.to_string()))?;

        *self.current_epoch.write().await = Some(epoch.clone());

        Ok(epoch)
    }

    /// Get a specific epoch by number
    pub async fn get_epoch(&self, epoch_number: u64) -> Result<Epoch, LighthouseError> {
        let url = format!("{}/v1/epoch/{}", self.config.http_url, epoch_number);

        let mut request = self.http_client.get(&url);
        if let Some(api_key) = &self.config.api_key {
            request = request.header("x-api-key", api_key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| LighthouseError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(LighthouseError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| LighthouseError::Parse(e.to_string()))
    }

    /// Get the cached current epoch (if available)
    pub async fn cached_epoch(&self) -> Option<Epoch> {
        self.current_epoch.read().await.clone()
    }

    /// Subscribe to epoch updates
    pub fn subscribe(&self) -> broadcast::Receiver<Epoch> {
        self.epoch_sender.subscribe()
    }

    /// Start the WebSocket connection for epoch subscriptions
    pub async fn start_subscription(&self) -> Result<(), LighthouseError> {
        let ws_url = &self.config.ws_url;
        let current_epoch = self.current_epoch.clone();
        let epoch_sender = self.epoch_sender.clone();

        info!("Connecting to Lighthouse WebSocket: {}", ws_url);

        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| LighthouseError::Connection(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();

        // Send subscription message
        write
            .send(Message::Text(
                r#"{"type":"subscribe","channel":"epochs"}"#.to_string(),
            ))
            .await
            .map_err(|e| LighthouseError::Connection(e.to_string()))?;

        info!("Subscribed to Lighthouse epochs");

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => match serde_json::from_str::<Epoch>(&text) {
                        Ok(epoch) => {
                            debug!("Received epoch {}", epoch.epoch_number);
                            *current_epoch.write().await = Some(epoch.clone());
                            let _ = epoch_sender.send(epoch);
                        }
                        Err(e) => {
                            warn!("Failed to parse epoch: {}", e);
                        }
                    },
                    Ok(Message::Ping(_)) => {
                        debug!("Received ping");
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket closed");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}


