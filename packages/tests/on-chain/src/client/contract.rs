pub mod bls;
pub mod ecdsa;
pub mod mirror;
pub mod mock;
pub mod trigger;

use layer_climb::pool::SigningClientPool;
use sdk::client::{WavsExecutor, WavsQuerier};

use crate::client::pool::TestPool;

#[derive(Clone)]
pub struct ContractTestClient {
    pub querier: WavsQuerier,
    pub executor: WavsExecutor,
}

impl ContractTestClient {
    pub async fn new() -> Self {
        let TestPool { pool, .. } = TestPool::get().await;
        Self {
            querier: pool.clone().into(),
            executor: pool.into(),
        }
    }

    pub fn pool(&self) -> SigningClientPool {
        match &self.executor {
            WavsExecutor::ClimbPool(pool) => pool.clone(),
            _ => unreachable!(),
        }
    }
}
