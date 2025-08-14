use async_trait::async_trait;
use cosmwasm_std::Uint64;
use crate::prelude::{WavsBasicQueryClientExt, WavsExecClientExt, WavsQueryClientExt, WavsServiceHandlerAddrExt};

#[async_trait(?Send)]
pub trait WavsMockQueryClientExt: WavsQueryClientExt {
    async fn get_service_handler_trigger_message(&self, trigger_id: Uint64) -> Result<String, cosmwasm_std::StdError> {
        let addr = WavsServiceHandlerAddrExt::addr(self.service_handler());
        self.service_handler().basic_contract_query(&addr, &mock_api::service_handler::QueryMsg::TriggerMessage { trigger_id })
            .await
    }
}

#[async_trait(?Send)]
pub trait WavsMockExecClientExt: WavsExecClientExt {
    async fn mock_exec_stuff(&self) {
        todo!()
    }
}