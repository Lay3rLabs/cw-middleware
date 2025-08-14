use layer_climb::prelude::ChainConfig;
use tokio::sync::OnceCell;
use utils::{config::ChainConfigs, path::repo_wavs_home};
use wavs_types::ChainName;

pub(super) const WAVS_BASE_PORT:u32 = 8123;
pub(super) const WAVS_AGGREGATOR_PORT:u32 =  8200;

// TODO - extend this for multiple operators
static TEST_CONFIG: OnceCell<TestConfig> = OnceCell::const_new();

#[derive(Clone)]
pub struct TestConfig {
    pub chain_name: ChainName,
    pub chain_config: ChainConfig,
}

impl TestConfig {
    pub async fn get() -> Self {
        TEST_CONFIG.get_or_init(Self::instantiate).await.clone()
    }

    pub fn wavs_endpoint(operator_number: Option<u32>) -> String {
        format!("http://localhost:{}", WAVS_BASE_PORT + operator_number.unwrap_or(0))
    }

    pub fn aggregator_endpoint() -> String {
        format!("http://localhost:{}", WAVS_AGGREGATOR_PORT)
    }

    async fn instantiate() -> Self {
        let chain_configs = ChainConfigs::load_from_wavs(repo_wavs_home())
            .await
            .expect("Failed to load chain configurations");

        let chain_name = ChainName::new(std::env::var("CHAIN_NAME").expect("CHAIN_NAME must be set")).unwrap();

        let chain_config = chain_configs
            .cosmos
            .get(&chain_name)
            .expect(&format!("No cosmos chain config found for {}", chain_name))
            .clone();

        tracing::info!("Using chain config for {}", chain_name);

        Self { chain_name, chain_config: chain_config.into() }
    }
}