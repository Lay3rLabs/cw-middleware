use cw_multi_test::{ContractWrapper, Executor};
use sdk::contract_kinds::mock::{
    MockServiceHandlerExecutor, MockServiceHandlerQuerier, MockServiceManagerExecutor,
    MockServiceManagerQuerier,
};
use sdk::service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier};
use sdk::service_manager::{ServiceManagerExecutor, ServiceManagerQuerier};
use shared_tests::wrapper::ContractTestWrapper;

use crate::client::simple_trigger::SimpleTriggerTestClient;
use crate::client::ContractTestClient;

#[derive(Clone)]
pub struct MockTestClient {
    pub service_handler_querier: MockServiceHandlerQuerier,
    pub service_handler_executor: MockServiceHandlerExecutor,
    pub service_manager_querier: MockServiceManagerQuerier,
    pub service_manager_executor: MockServiceManagerExecutor,
}

impl MockTestClient {
    pub fn new(client: ContractTestClient) -> Self {
        let app = client.app();
        let admin = client.admin();

        let contract = ContractWrapper::new(
            mock_service_manager::entry::execute,
            mock_service_manager::entry::instantiate,
            mock_service_manager::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let service_manager = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin.clone(),
                &mock_api::service_manager::InstantiateMsg {},
                &[],
                "Mock Service Manager",
                None,
            )
            .unwrap();

        let contract = ContractWrapper::new(
            mock_service_handler::entry::execute,
            mock_service_handler::entry::instantiate,
            mock_service_handler::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let service_handler = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin,
                &mock_api::service_handler::InstantiateMsg {
                    service_manager: service_manager.to_string(),
                },
                &[],
                "Mock Service Handler",
                None,
            )
            .unwrap();

        let service_handler_querier = MockServiceHandlerQuerier::new(ServiceHandlerQuerier::new(
            client.querier.clone(),
            service_handler.clone(),
        ));
        let service_handler_executor = MockServiceHandlerExecutor::new(
            ServiceHandlerExecutor::new(client.executor.clone(), service_handler),
        );
        let service_manager_querier = MockServiceManagerQuerier::new(ServiceManagerQuerier::new(
            client.querier.clone(),
            service_manager.clone(),
        ));
        let service_manager_executor = MockServiceManagerExecutor::new(
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
