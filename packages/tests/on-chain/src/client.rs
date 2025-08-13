use std::future::Future;
use std::pin::Pin;
use utils::client::on_chain::WavsSigningPoolClient;
use utils::prelude::*;
use wavs_types::{Service, ServiceManager};

use crate::client::{config::TestConfig, node::WavsNodeClient};

pub mod contract;
pub mod node;
pub mod config;
pub(super) mod pool;

pub struct TestClientBuilder {
    service_maker: Option<Box<dyn Fn(&WavsSigningPoolClient, ServiceManager) -> Pin<Box<dyn Future<Output = Service> + Send>>>>,
}

impl TestClientBuilder {
    pub fn new() -> Self {
        Self {
            service_maker: None 
        }
    }

    pub fn with_service_maker<A, B>(mut self, service_maker: A) -> Self
    where
        A: Fn(&WavsSigningPoolClient, ServiceManager) -> B + 'static,
        B: Future<Output = Service> + Send + 'static,
    {
        self.service_maker = Some(Box::new(move |client, manager| {
            Box::pin(service_maker(client, manager))
        }));
        self
    }

    pub async fn build(self) -> TestClient {
        let contracts = contract::new_mock_client().await;
        let config = TestConfig::get().await;
        let node = WavsNodeClient::new(None).await;

        let service_maker = self.service_maker.expect("Service maker function must be provided");

        let service_manager_address = config.chain_config.parse_address(contracts.service_manager_querier().addr().as_str()).unwrap();

        let service_manager = ServiceManager::Cosmos {
            chain_name: config.chain_name.clone(),
            address: service_manager_address
        };

        let service = (service_maker)(&contracts, service_manager).await;

        TestClient {
            contracts,
            node,
            service
        }
    }
}

pub struct TestClient {
    pub contracts: WavsSigningPoolClient,
    pub node: WavsNodeClient,
    pub service: Service,
}