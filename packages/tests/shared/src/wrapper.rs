use sdk::{
    contract_kinds::simple_trigger::{SimpleTriggerExecutor, SimpleTriggerQuerier},
    service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier},
    service_manager::{ServiceManagerExecutor, ServiceManagerQuerier},
};

#[derive(Clone)]
pub struct ContractTestWrapper {
    pub service_handler_querier: ServiceHandlerQuerier,
    pub service_handler_executor: ServiceHandlerExecutor,
    pub service_manager_querier: ServiceManagerQuerier,
    pub service_manager_executor: ServiceManagerExecutor,
    pub simple_trigger_querier: SimpleTriggerQuerier,
    pub simple_trigger_executor: SimpleTriggerExecutor,
}

impl ContractTestWrapper {
    pub fn new(
        service_handler_querier: ServiceHandlerQuerier,
        service_handler_executor: ServiceHandlerExecutor,
        service_manager_querier: ServiceManagerQuerier,
        service_manager_executor: ServiceManagerExecutor,
        simple_trigger_querier: SimpleTriggerQuerier,
        simple_trigger_executor: SimpleTriggerExecutor,
    ) -> Self {
        Self {
            service_handler_querier,
            service_handler_executor,
            service_manager_querier,
            service_manager_executor,
            simple_trigger_querier,
            simple_trigger_executor,
        }
    }
}
