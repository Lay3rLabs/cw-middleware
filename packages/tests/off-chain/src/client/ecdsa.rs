use cw_multi_test::{ContractWrapper, Executor};
use sdk::contract_kinds::ecdsa::{
    EcdsaServiceHandlerExecutor, EcdsaServiceHandlerQuerier, EcdsaServiceManagerExecutor,
    EcdsaServiceManagerQuerier,
};
use sdk::service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier};
use sdk::service_manager::{ServiceManagerExecutor, ServiceManagerQuerier};
use shared_tests::wrapper::ContractTestWrapper;

use crate::client::simple_trigger::SimpleTriggerTestClient;
use crate::client::ContractTestClient;

#[derive(Clone)]
pub struct EcdsaTestClient {
    pub service_handler_querier: EcdsaServiceHandlerQuerier,
    pub service_handler_executor: EcdsaServiceHandlerExecutor,
    pub service_manager_querier: EcdsaServiceManagerQuerier,
    pub service_manager_executor: EcdsaServiceManagerExecutor,
}

impl EcdsaTestClient {
    pub fn new(client: ContractTestClient) -> Self {
        let app = client.app();
        let admin = client.admin();

        let contract = ContractWrapper::new(
            ecdsa_service_manager::entry::execute,
            ecdsa_service_manager::entry::instantiate,
            ecdsa_service_manager::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let service_manager = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin.clone(),
                &ecdsa_api::service_manager::InstantiateMsg {},
                &[],
                "ECDSA Service Manager",
                None,
            )
            .unwrap();

        let contract = ContractWrapper::new(
            ecdsa_service_handler::entry::execute,
            ecdsa_service_handler::entry::instantiate,
            ecdsa_service_handler::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let service_handler = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin,
                &ecdsa_api::service_handler::InstantiateMsg {
                    service_manager: service_manager.to_string(),
                },
                &[],
                "ECDSA Service Handler",
                None,
            )
            .unwrap();

        let service_handler_querier = EcdsaServiceHandlerQuerier::new(ServiceHandlerQuerier::new(
            client.querier.clone(),
            service_handler.clone(),
        ));
        let service_handler_executor = EcdsaServiceHandlerExecutor::new(
            ServiceHandlerExecutor::new(client.executor.clone(), service_handler),
        );
        let service_manager_querier = EcdsaServiceManagerQuerier::new(ServiceManagerQuerier::new(
            client.querier.clone(),
            service_manager.clone(),
        ));
        let service_manager_executor = EcdsaServiceManagerExecutor::new(
            ServiceManagerExecutor::new(client.executor.clone(), service_manager),
        );
        Self {
            service_handler_querier,
            service_handler_executor,
            service_manager_querier,
            service_manager_executor,
        }
    }

    pub fn wrap_test(&self, simple_trigger: &SimpleTriggerTestClient) -> ContractTestWrapper {
        ContractTestWrapper {
            service_handler_querier: self.service_handler_querier.service_handler().clone(),
            service_handler_executor: self.service_handler_executor.service_handler().clone(),
            service_manager_querier: self.service_manager_querier.service_manager().clone(),
            service_manager_executor: self.service_manager_executor.service_manager().clone(),
            simple_trigger_querier: simple_trigger.querier.clone(),
            simple_trigger_executor: simple_trigger.executor.clone(),
        }
    }
}
