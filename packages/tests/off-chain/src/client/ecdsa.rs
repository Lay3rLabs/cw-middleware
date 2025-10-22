use cw_multi_test::{ContractWrapper, Executor};
use cw_wavs_sdk::contract_kinds::ecdsa::{
    EcdsaServiceHandlerExecutor, EcdsaServiceHandlerQuerier, EcdsaServiceManagerExecutor,
    EcdsaServiceManagerQuerier,
};
use cw_wavs_sdk::service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier};
use cw_wavs_sdk::service_manager::{ServiceManagerExecutor, ServiceManagerQuerier};
use shared_tests::wrapper::ContractTestWrapper;

use crate::client::trigger::SimpleTriggerTestClient;
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
            cw_wavs_ecdsa_service_manager::entry::execute,
            cw_wavs_ecdsa_service_manager::entry::instantiate,
            cw_wavs_ecdsa_service_manager::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let service_manager = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin.clone(),
                &cw_wavs_ecdsa_api::service_manager::InstantiateMsg {},
                &[],
                "ECDSA Service Manager",
                None,
            )
            .unwrap();

        let contract = ContractWrapper::new(
            cw_wavs_ecdsa_service_handler::entry::execute,
            cw_wavs_ecdsa_service_handler::entry::instantiate,
            cw_wavs_ecdsa_service_handler::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let service_handler = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin,
                &cw_wavs_ecdsa_api::service_handler::InstantiateMsg {
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

    pub fn wrap_test(
        &self,
        cw_wavs_trigger_simple: &SimpleTriggerTestClient,
    ) -> ContractTestWrapper {
        ContractTestWrapper {
            service_handler_querier: self.service_handler_querier.service_handler().clone(),
            service_handler_executor: self.service_handler_executor.service_handler().clone(),
            service_manager_querier: self.service_manager_querier.service_manager().clone(),
            service_manager_executor: self.service_manager_executor.service_manager().clone(),
            cw_wavs_trigger_simple_querier: cw_wavs_trigger_simple.querier.clone(),
            cw_wavs_trigger_simple_executor: cw_wavs_trigger_simple.executor.clone(),
        }
    }
}
