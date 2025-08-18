use cw_multi_test::{ContractWrapper, Executor};
use sdk::contract_kinds::trigger::{SimpleTriggerExecutor, SimpleTriggerQuerier};

use crate::client::ContractTestClient;

#[derive(Clone)]
pub struct SimpleTriggerTestClient {
    pub querier: SimpleTriggerQuerier,
    pub executor: SimpleTriggerExecutor,
}

impl SimpleTriggerTestClient {
    pub fn new(client: ContractTestClient) -> Self {
        let app = client.app();
        let admin = client.admin();

        let contract = ContractWrapper::new(
            trigger_simple::entry::execute,
            trigger_simple::entry::instantiate,
            trigger_simple::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let address = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin.clone(),
                &bls_api::service_manager::InstantiateMsg {},
                &[],
                "BLS Service Manager",
                None,
            )
            .unwrap();

        let querier = SimpleTriggerQuerier::new(client.querier.clone(), address.clone());
        let executor = SimpleTriggerExecutor::new(client.executor.clone(), address);

        Self { querier, executor }
    }
}
