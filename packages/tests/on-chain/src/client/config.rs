use layer_climb::prelude::ChainConfig;
use tokio::sync::OnceCell;
use utils::{config::ChainConfigs, path::repo_wavs_home};
use wavs_types::ChainName;

pub(super) const WAVS_BASE_PORT:u32 = 8123;
pub(super) const WAVS_AGGREGATOR_PORT:u32 =  8200;

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

    async fn instantiate() -> Self {
        let chain_configs = ChainConfigs::load_from_wavs(repo_wavs_home())
            .await
            .expect("Failed to load chain configurations");

        let (chain_name, chain_config) = chain_configs
            .cosmos
            .into_iter()
            .next()
            .expect("No cosmos chain config found");

        Self { chain_name, chain_config: chain_config.into() }
    }
}