use sdk::contract_kinds::trigger::{SimpleTriggerExecutor, SimpleTriggerQuerier};

use crate::client::code_ids::CodeId;
use crate::client::contract::ContractTestClient;

#[derive(Clone)]
pub struct SimpleTriggerTestClient {
    pub querier: SimpleTriggerQuerier,
    pub executor: SimpleTriggerExecutor,
}

impl SimpleTriggerTestClient {
    pub async fn new(test_client: ContractTestClient) -> Self {
        let pool = test_client.pool();
        let client = pool.get().await.unwrap();

        let (address, _) = client
            .contract_instantiate(
                None,
                CodeId::new_trigger_simple().await,
                "Simple Trigger",
                &trigger_api::simple::InstantiateMsg {},
                vec![],
                None,
            )
            .await
            .unwrap();

        let querier = SimpleTriggerQuerier::new(
            test_client.querier.clone(),
            address.clone().try_into().unwrap(),
        );
        let executor =
            SimpleTriggerExecutor::new(client.clone().into(), address.clone().try_into().unwrap());

        Self { querier, executor }
    }
}
