use sdk::contract_kinds::bls::{
    BlsServiceHandlerExecutor, BlsServiceHandlerQuerier, BlsServiceManagerExecutor,
    BlsServiceManagerQuerier,
};
use sdk::service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier};
use sdk::service_manager::{ServiceManagerExecutor, ServiceManagerQuerier};
use shared_tests::wrapper::ContractTestWrapper;

use crate::client::code_ids::CodeId;
use crate::client::contract::trigger::SimpleTriggerTestClient;
use crate::client::contract::ContractTestClient;

#[derive(Clone)]
pub struct BlsTestClient {
    pub service_handler_querier: BlsServiceHandlerQuerier,
    pub service_handler_executor: BlsServiceHandlerExecutor,
    pub service_manager_querier: BlsServiceManagerQuerier,
    pub service_manager_executor: BlsServiceManagerExecutor,
}

impl BlsTestClient {
    pub async fn new(test_client: ContractTestClient) -> Self {
        let pool = test_client.pool();
        let client = pool.get().await.unwrap();

        let (service_manager, _) = client
            .contract_instantiate(
                None,
                CodeId::new_bls_service_manager().await,
                "BLS Service Manager",
                &bls_api::service_manager::InstantiateMsg {},
                vec![],
                None,
            )
            .await
            .unwrap();

        let (service_handler, _) = client
            .contract_instantiate(
                None,
                CodeId::new_bls_service_handler().await,
                "BLS Service Handler",
                &bls_api::service_handler::InstantiateMsg {
                    service_manager: service_manager.to_string(),
                },
                vec![],
                None,
            )
            .await
            .unwrap();

        let service_handler_querier = BlsServiceHandlerQuerier::new(ServiceHandlerQuerier::new(
            test_client.querier.clone(),
            service_handler.clone().try_into().unwrap(),
        ));
        let service_handler_executor = BlsServiceHandlerExecutor::new(ServiceHandlerExecutor::new(
            client.clone().into(),
            service_handler.try_into().unwrap(),
        ));
        let service_manager_querier = BlsServiceManagerQuerier::new(ServiceManagerQuerier::new(
            test_client.querier.clone(),
            service_manager.clone().try_into().unwrap(),
        ));
        let service_manager_executor = BlsServiceManagerExecutor::new(ServiceManagerExecutor::new(
            client.clone().into(),
            service_manager.try_into().unwrap(),
        ));
        Self {
            service_handler_querier,
            service_handler_executor,
            service_manager_querier,
            service_manager_executor,
        }
    }

    pub fn wrap_test(&self, trigger_simple: &SimpleTriggerTestClient) -> ContractTestWrapper {
        ContractTestWrapper {
            service_handler_querier: self.service_handler_querier.service_handler().clone(),
            service_handler_executor: self.service_handler_executor.service_handler().clone(),
            service_manager_querier: self.service_manager_querier.service_manager().clone(),
            service_manager_executor: self.service_manager_executor.service_manager().clone(),
            trigger_simple_querier: trigger_simple.querier.clone(),
            trigger_simple_executor: trigger_simple.executor.clone(),
        }
    }
}
