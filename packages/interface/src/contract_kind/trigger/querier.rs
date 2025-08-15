use async_trait::async_trait;
use cosmwasm_std::Uint64;
use trigger_api::simple::QueryMsg;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use super::HasSimpleTriggerAddr;
use crate::QueryClientExt;

#[async_trait(?Send)]
pub trait SimpleTriggerQueryClient: QueryClientExt + HasSimpleTriggerAddr
{
    async fn simple_trigger_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = HasSimpleTriggerAddr::addr(self);
        self.contract_query(&contract_addr, msg).await
    }

    async fn get_trigger_message(
        &self,
        trigger_id: impl Into<Uint64>,
    ) -> Result<String, cosmwasm_std::StdError> {
        self.simple_trigger_query(&trigger_api::simple::QueryMsg::TriggerMessage {
            trigger_id: trigger_id.into(),
        })
        .await
    }
}

impl <T> SimpleTriggerQueryClient for T
where
    T: QueryClientExt + HasSimpleTriggerAddr { }