use anyhow::{Context, Result};
use layer_climb::prelude::*;
use tokio::sync::OnceCell;
use utils::{faucet, path::repo_root};
use wavs_types_full::{
    aggregator::RegisterServiceRequest, AddServiceRequest, ComponentDigest, GetSignerRequest,
    SaveServiceResponse, Service, ServiceManager, SignerResponse, UploadComponentResponse,
};

use crate::client::config::TestConfig;

// TODO - register the digest for each additional operator
static OPERATOR_COMPONENT_DIGEST: OnceCell<ComponentDigest> = OnceCell::const_new();
static AGGREGATOR_COMPONENT_DIGEST: OnceCell<ComponentDigest> = OnceCell::const_new();
static CLIENT: OnceCell<WavsNodeClient> = OnceCell::const_new();

#[derive(Clone)]
pub struct WavsNodeClient {
    inner: reqwest::Client,
}

impl WavsNodeClient {
    pub async fn new() -> Self {
        CLIENT.get_or_init(Self::instantiate).await.clone()
    }

    async fn instantiate() -> Self {
        // make sure the aggregator has funds to submit on-chain
        let chain_config = TestConfig::get().await.chain_config;
        let chain_config: ChainConfig = chain_config;
        let querier = QueryClient::new(chain_config.clone(), None).await.unwrap();

        let mnemonic = std::env::var("WAVS_AGGREGATOR_COSMOS_MNEMONIC")
            .expect("WAVS_AGGREGATOR_COSMOS_MNEMONIC must be set");
        let signer = KeySigner::new_mnemonic_str(&mnemonic, None)
            .expect("Failed to create KeySigner from aggregator mnemonic");

        let addr = chain_config
            .address_from_pub_key(&signer.public_key().await.unwrap())
            .unwrap();

        let balance = querier
            .balance(addr.clone(), None)
            .await
            .unwrap()
            .unwrap_or_default();

        if balance < 10000000000 {
            tracing::info!(
                "aggregator {} has balance of {}, tapping faucet...",
                addr,
                balance
            );
            faucet::tap(&addr, &chain_config.gas_denom).await.unwrap();
            let new_balance = querier
                .balance(addr, None)
                .await
                .unwrap()
                .unwrap_or_default();
            if new_balance == balance {
                panic!("Failed to tap faucet, balance did not change");
            }
            tracing::info!("new balance is {:?}", new_balance);
        } else {
            tracing::info!(
                "aggregator {} has balance of {}, no need to tap faucet",
                addr,
                balance
            );
        }

        Self {
            inner: reqwest::Client::new(),
        }
    }

    pub async fn operator_component_digest() -> ComponentDigest {
        OPERATOR_COMPONENT_DIGEST
            .get_or_init(upload_operator_component_digest)
            .await
            .clone()
    }

    pub async fn aggregator_component_digest() -> ComponentDigest {
        AGGREGATOR_COMPONENT_DIGEST
            .get_or_init(upload_aggregator_component_digest)
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

    pub async fn get_service_signing_key_addr(&self, service: &Service) -> anyhow::Result<EvmAddr> {
        let body = serde_json::to_string(&GetSignerRequest {
            service_manager: service.manager.clone(),
        })?;

        let url = format!("{}/service-key", TestConfig::wavs_endpoint(None));

        let response: SignerResponse = self
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

        let SignerResponse::Secp256k1 { evm_address, .. } = response;

        evm_address.parse()
    }
}

async fn upload_operator_component_digest() -> ComponentDigest {
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

async fn upload_aggregator_component_digest() -> ComponentDigest {
    let wasm_path = repo_root()
        .unwrap()
        .join("packages")
        .join("components")
        .join("artifacts")
        .join("TODO.wasm")
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
