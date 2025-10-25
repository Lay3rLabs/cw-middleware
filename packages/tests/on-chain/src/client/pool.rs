use deadpool::managed::Pool;
use tokio::sync::OnceCell;

use layer_climb::{
    pool::{SigningClientPool, SigningClientPoolManager},
    prelude::*,
};
use utils::faucet;

use crate::client::config::TestConfig;

const MAINTAIN_MINIMUM_BALANCE_THRESHOLD: u128 = 100000;
const MAINTAIN_MINIMUM_BALANCE_TOPUP: u128 = 10000000;

static TEST_POOL: OnceCell<TestPool> = OnceCell::const_new();

#[derive(Clone)]
pub struct TestPool {
    pub pool: SigningClientPool,
}

impl TestPool {
    pub async fn get() -> Self {
        TEST_POOL.get_or_init(TestPool::instantiate).await.clone()
    }

    async fn instantiate() -> Self {
        let mnemonic = std::env::var("CLI_MNEMONIC").expect("CLI_MNEMONIC must be set");

        let chain_config = TestConfig::get().await.chain_config;
        let chain_config: ChainConfig = chain_config;
        let querier = QueryClient::new(chain_config.clone(), None).await.unwrap();

        // Before we run off and create the pool, make sure it has funds!
        let signer = KeySigner::new_mnemonic_str(&mnemonic, None)
            .expect("Failed to create KeySigner from mnemonic");

        let addr = chain_config
            .address_from_pub_key(&signer.public_key().await.unwrap())
            .unwrap();

        let balance = querier
            .balance(addr.clone(), None)
            .await
            .unwrap()
            .unwrap_or_default();

        if balance < 10000000000 {
            tracing::info!("{} has balance of {}, tapping faucet...", addr, balance);
            faucet::tap(&addr, &chain_config.gas_denom, None)
                .await
                .unwrap();
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
            tracing::info!("{} has balance of {}, no need to tap faucet", addr, balance);
        }

        // now we can properly create the pool
        let pool =
            SigningClientPoolManager::new_mnemonic(mnemonic, chain_config.clone(), None, None)
                .with_minimum_balance(
                    MAINTAIN_MINIMUM_BALANCE_THRESHOLD,
                    MAINTAIN_MINIMUM_BALANCE_TOPUP,
                    None,
                    None,
                )
                .await
                .unwrap();

        let pool = SigningClientPool::new(Pool::builder(pool).max_size(8).build().unwrap());

        Self { pool }
    }
}
