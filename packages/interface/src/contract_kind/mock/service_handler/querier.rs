use async_trait::async_trait;
use cosmwasm_std::Uint64;
use mock_api::service_handler::{QueryMsg, TriggerMessageResponse};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use super::HasMockServiceHandlerAddr;

use crate::ServiceHandlerQueryClient;

#[async_trait(?Send)]
pub trait MockServiceHandlerQueryClient: ServiceHandlerQueryClient + HasMockServiceHandlerAddr
{
    async fn mock_service_handler_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = HasMockServiceHandlerAddr::addr(self);
        self.contract_query(&contract_addr, msg).await
    }

    async fn get_handled_trigger_message(
        &self,
        trigger_id: Uint64,
    ) -> Result<String, cosmwasm_std::StdError> {
        let resp: TriggerMessageResponse = self
            .mock_service_handler_query(&mock_api::service_handler::QueryMsg::TriggerMessage {
                trigger_id,
            })
            .await?;

        Ok(resp.message)
    }
}

impl <T> MockServiceHandlerQueryClient for T
where
    T: ServiceHandlerQueryClient + HasMockServiceHandlerAddr { }