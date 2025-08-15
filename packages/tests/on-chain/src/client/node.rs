use anyhow::{Context, Result};
use tokio::sync::OnceCell;
use utils::path::repo_root;
use wavs_types::{
    aggregator::RegisterServiceRequest, AddServiceRequest, ComponentDigest, SaveServiceResponse,
    Service, ServiceManager, UploadComponentResponse,
};

use crate::client::config::TestConfig;

// TODO - extend this for multiple operators
static COMPONENT_DIGEST: OnceCell<ComponentDigest> = OnceCell::const_new();

pub struct WavsNodeClient {
    inner: reqwest::Client,
}

impl WavsNodeClient {
    pub async fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }

    pub async fn component_digest() -> ComponentDigest {
        COMPONENT_DIGEST
            .get_or_init(upload_component_digest)
            .await
            .clone()
    }

    pub async fn save_service_url(&self, service: &Service) -> Result<String> {
        let body = serde_json::to_string(service)?;

        let url = format!("{}/save-service", TestConfig::wavs_endpoint(None));
        let response: SaveServiceResponse = self
            .inner
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .with_context(|| format!("Failed to send request to {url}"))?
            .json()
            .await
            .with_context(|| format!("Failed to parse response from {url}"))?;

        Ok(format!(
            "{}/service-by-hash/{}",
            TestConfig::wavs_endpoint(None),
            response.hash
        ))
    }

    pub async fn deploy_service(&self, service_manager: ServiceManager) -> Result<()> {
        let body: String = serde_json::to_string(&AddServiceRequest {
            service_manager: service_manager.clone(),
        })?;

        let url = format!("{}/app", TestConfig::wavs_endpoint(None));
        let response = self
            .inner
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .with_context(|| format!("Failed to send request to {url}"))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "<Failed to read response body>".to_string());

            anyhow::bail!("{} from {}: {}", status, url, error_text);
        }

        Ok(())
    }

    pub async fn register_aggregator_service(&self, service: &Service) -> anyhow::Result<()> {
        let endpoint = format!("{}/register-service", TestConfig::aggregator_endpoint());
        let payload = RegisterServiceRequest {
            service_manager: service.manager.clone(),
        };

        tracing::info!(
            "Registering service {} with aggregator at {}",
            service.id(),
            endpoint
        );

        self.inner
            .post(&endpoint)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

async fn upload_component_digest() -> ComponentDigest {
    let wasm_path = repo_root()
        .unwrap()
        .join("packages")
        .join("components")
        .join("artifacts")
        .join("echo_with_id.wasm")
        .to_path_buf();

    let wasm_bytes = tokio::fs::read(&wasm_path)
        .await
        .unwrap_or_else(|_| panic!("Failed to read {}", wasm_path.display()));

    let response: UploadComponentResponse = reqwest::Client::new()
        .post(format!("{}/upload", TestConfig::wavs_endpoint(None)))
        .body(wasm_bytes)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    response.digest
}
