use std::sync::Arc;

use sdk::{
    service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier},
    service_manager::{ServiceManagerExecutor, ServiceManagerQuerier},
};

use crate::client::{
    config::TestConfig,
    contract::{
        bls::BlsTestClient, ecdsa::EcdsaTestClient, mirror::MirrorTestClient, mock::MockTestClient,
        trigger::SimpleTriggerTestClient, ContractTestClient,
    },
    node::WavsNodeClient,
};

#[derive(Clone)]
pub struct TestClient {
    pub node: Arc<WavsNodeClient>,
    pub service: TestService,
    pub trigger: SimpleTriggerTestClient,
    pub config: Arc<TestConfig>,
}

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum TestService {
    Mock(MockTestClient),
    Ecdsa(EcdsaTestClient),
    Bls(BlsTestClient),
    Mirror(MirrorTestClient),
}

impl TestService {
    pub fn wavs_service_handler_querier(&self) -> ServiceHandlerQuerier {
        match self {
            Self::Mock(client) => client.service_handler_querier.service_handler().clone(),
            Self::Ecdsa(client) => client.service_handler_querier.service_handler().clone(),
            Self::Bls(client) => client.service_handler_querier.service_handler().clone(),
            Self::Mirror(client) => client.service_handler_querier.service_handler().clone(),
        }
    }

    pub fn wavs_service_handler_executor(&self) -> ServiceHandlerExecutor {
        match self {
            Self::Mock(client) => client.service_handler_executor.service_handler().clone(),
            Self::Ecdsa(client) => client.service_handler_executor.service_handler().clone(),
            Self::Bls(client) => client.service_handler_executor.service_handler().clone(),
            Self::Mirror(client) => client.service_handler_executor.service_handler().clone(),
        }
    }

    pub fn wavs_service_manager_querier(&self) -> ServiceManagerQuerier {
        match self {
            Self::Mock(client) => client.service_manager_querier.service_manager().clone(),
            Self::Ecdsa(client) => client.service_manager_querier.service_manager().clone(),
            Self::Bls(client) => client.service_manager_querier.service_manager().clone(),
            Self::Mirror(client) => client.service_manager_querier.service_manager().clone(),
        }
    }

    pub fn wavs_service_manager_executor(&self) -> ServiceManagerExecutor {
        match self {
            Self::Mock(client) => client.service_manager_executor.service_manager().clone(),
            Self::Ecdsa(client) => client.service_manager_executor.service_manager().clone(),
            Self::Bls(client) => client.service_manager_executor.service_manager().clone(),
            Self::Mirror(client) => client.service_manager_executor.service_manager().clone(),
        }
    }
}

impl TestClient {
    pub async fn new_mock() -> Self {
        let client = ContractTestClient::new().await;
        let trigger = SimpleTriggerTestClient::new(client.clone()).await;
        let service = TestService::Mock(MockTestClient::new(client.clone()).await);
        let config = TestConfig::get().await;
        let node = WavsNodeClient::new().await;

        Self {
            node: Arc::new(node),
            trigger,
            service,
            config: Arc::new(config),
        }
    }

    pub async fn new_ecdsa() -> Self {
        let client = ContractTestClient::new().await;
        let trigger = SimpleTriggerTestClient::new(client.clone()).await;
        let service = TestService::Ecdsa(EcdsaTestClient::new(client.clone()).await);
        let config = TestConfig::get().await;
        let node = WavsNodeClient::new().await;

        Self {
            node: Arc::new(node),
            trigger,
            service,
            config: Arc::new(config),
        }
    }

    pub async fn new_bls() -> Self {
        let client = ContractTestClient::new().await;
        let trigger = SimpleTriggerTestClient::new(client.clone()).await;
        let service = TestService::Bls(BlsTestClient::new(client.clone()).await);
        let config = TestConfig::get().await;
        let node = WavsNodeClient::new().await;

        Self {
            node: Arc::new(node),
            trigger,
            service,
            config: Arc::new(config),
        }
    }

    pub async fn new_mirror() -> Self {
        let client = ContractTestClient::new().await;
        let trigger = SimpleTriggerTestClient::new(client.clone()).await;
        let service = TestService::Mirror(MirrorTestClient::new(client.clone()).await);
        let config = TestConfig::get().await;
        let node = WavsNodeClient::new().await;

        Self {
            node: Arc::new(node),
            trigger,
            service,
            config: Arc::new(config),
        }
    }
}
