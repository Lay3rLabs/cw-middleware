use wavs_types::{SaveServiceResponse, Service};
use anyhow::{Context, Result};

use crate::client::config::WAVS_AGGREGATOR_PORT;



pub struct WavsNodeClient {
    pub endpoint: String,
    inner: reqwest::Client,
}

impl WavsNodeClient {
    pub async fn new(operator_index: Option<u32>) -> Self {
        let wavs_port = WAVS_AGGREGATOR_PORT + operator_index.unwrap_or_default(); 
        // Initialize the WavsNodeClient here if needed
        Self {
            endpoint: format!("http://localhost:{}", wavs_port),
            inner: reqwest::Client::new(),
        }
    }

    pub async fn save_service_url(&self, service: &Service) -> Result<String> {
        let body = serde_json::to_string(service)?;

        let url = format!("{}/save-service", self.endpoint);
        let response: SaveServiceResponse = self
            .inner
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .with_context(|| format!("Failed to send request to {}", url))?
            .json()
            .await
            .with_context(|| format!("Failed to parse response from {}", url))?;

        Ok(format!(
            "{}/service-by-hash/{}",
            self.endpoint, response.hash
        ))
    }
}