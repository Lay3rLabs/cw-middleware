use crate::client::{
    config::TestConfig,
    contract::{BlsContractClient, EcdsaContractClient, MockContractClient},
    node::WavsNodeClient,
};
use layer_climb::prelude::Address;
use mock_api::trigger::PushMessageEvent;
use std::{collections::BTreeMap, sync::Arc};
use utils::{contract_client::on_chain::WavsSigningPoolClient, prelude::*};
use wavs_types::{
    Component, ComponentSource, CosmosContractSubmission, Service, ServiceManager, Submit, Trigger,
    Workflow,
};

pub mod config;
pub mod contract;
pub mod node;
pub(super) mod pool;

#[derive(Clone)]
pub struct TestClient {
    pub node: Arc<WavsNodeClient>,
    pub contract: TestContractClient,
    pub config: Arc<TestConfig>,
}

#[derive(Clone)]
pub enum TestContractClient {
    Mock(MockContractClient),
    Ecdsa(EcdsaContractClient),
    Bls(BlsContractClient),
}

impl TestClient {
    pub async fn new_mock() -> Self {
        let contracts = MockContractClient::new().await;
        let config = TestConfig::get().await;
        let node = WavsNodeClient::new(config.clone()).await;

        Self {
            node: Arc::new(node),
            contract: TestContractClient::Mock(contracts),
            config: Arc::new(config),
        }
    }

    pub async fn new_ecdsa() -> Self {
        let contracts = EcdsaContractClient::new().await;
        let config = TestConfig::get().await;
        let node = WavsNodeClient::new(config.clone()).await;

        Self {
            node: Arc::new(node),
            contract: TestContractClient::Ecdsa(contracts),
            config: Arc::new(config),
        }
    }

    pub async fn new_bls() -> Self {
        let contracts = BlsContractClient::new().await;
        let config = TestConfig::get().await;
        let node = WavsNodeClient::new(config.clone()).await;

        Self {
            node: Arc::new(node),
            contract: TestContractClient::Bls(contracts),
            config: Arc::new(config),
        }
    }

    pub async fn new_service(&self) -> Service {
        let component = WavsNodeClient::component_digest().await;

        let mut workflows = BTreeMap::new();

        workflows.insert(
            "messenger".parse().unwrap(),
            Workflow {
                trigger: Trigger::CosmosContractEvent {
                    address: self.trigger_address(),
                    chain_name: self.config.chain_name.clone(),
                    event_type: PushMessageEvent::EVENT_TYPE.to_string(),
                },
                component: Component::new(ComponentSource::Digest(component)),
                submit: Submit::Aggregator {
                    url: TestConfig::aggregator_endpoint(),
                    component: None,
                    evm_contracts: None,
                    cosmos_contracts: Some(vec![CosmosContractSubmission::new(
                        self.config.chain_name.clone(),
                        self.service_handler_address(),
                        None,
                    )]),
                },
            },
        );

        Service {
            name: "test".to_string(),
            status: wavs_types::ServiceStatus::Paused,
            workflows,
            manager: ServiceManager::Cosmos {
                chain_name: self.config.chain_name.clone(),
                address: self.service_manager_address(),
            },
        }
    }

    pub fn service_handler_address(&self) -> Address {
        let addr = self.query_client().service_handler_querier().addr();
        self.config
            .chain_config
            .parse_address(addr.as_str())
            .unwrap()
    }

    pub fn service_manager_address(&self) -> Address {
        let addr = self.query_client().service_manager_querier().addr();
        self.config
            .chain_config
            .parse_address(addr.as_str())
            .unwrap()
    }

    pub fn trigger_address(&self) -> Address {
        let addr = self.query_client().trigger_querier().addr();
        self.config
            .chain_config
            .parse_address(addr.as_str())
            .unwrap()
    }
}

impl HasWavsQueryClient for TestClient {
    type QueryClient = WavsSigningPoolClient;

    fn query_client(&self) -> &Self::QueryClient {
        match &self.contract {
            TestContractClient::Mock(client) => &client.query_client(),
            TestContractClient::Ecdsa(client) => &client.query_client(),
            TestContractClient::Bls(client) => &client.query_client(),
        }
    }
}

impl HasWavsExecClient for TestClient {
    type ExecClient = WavsSigningPoolClient;

    fn exec_client(&self) -> &Self::ExecClient {
        match &self.contract {
            TestContractClient::Mock(client) => &client.exec_client(),
            TestContractClient::Ecdsa(client) => &client.exec_client(),
            TestContractClient::Bls(client) => &client.exec_client(),
        }
    }
}
