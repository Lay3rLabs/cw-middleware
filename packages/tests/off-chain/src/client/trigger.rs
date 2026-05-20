use cw_multi_test::{ContractWrapper, Executor};
use cw_wavs_sdk::contract_kinds::trigger::{SimpleTriggerExecutor, SimpleTriggerQuerier};

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
            cw_wavs_trigger_simple::entry::execute,
            cw_wavs_trigger_simple::entry::instantiate,
            cw_wavs_trigger_simple::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let address = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin.clone(),
                &cw_wavs_trigger_api::simple::InstantiateMsg::default(),
                &[],
                "Simple Trigger",
                None,
            )
            .unwrap();

        let querier = SimpleTriggerQuerier::new(client.querier.clone(), address.clone());
        let executor = SimpleTriggerExecutor::new(client.executor.clone(), address);

        Self { querier, executor }
    }
}
