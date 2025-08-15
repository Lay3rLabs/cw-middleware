use async_trait::async_trait;
use cosmwasm_std::{Coin, Uint64};
use mock_api::trigger::PushMessageEvent;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::prelude::{
    TxResponseExt, WavsBasicExecClientExt, WavsBasicQueryClientExt, WavsTriggerAddrExt,
};

// Trigger Query
#[async_trait(?Send)]
pub trait WavsTriggerQueryClientExt: WavsBasicQueryClientExt + WavsTriggerAddrExt {
    async fn query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &mock_api::trigger::QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = self.addr();
        self.basic_contract_query(&contract_addr, msg).await
    }

    async fn get_trigger_message(
        &self,
        trigger_id: impl Into<Uint64>,
    ) -> Result<String, cosmwasm_std::StdError> {
        self.query(&mock_api::trigger::QueryMsg::TriggerMessage {
            trigger_id: trigger_id.into(),
        })
        .await
    }
}

// Trigger Exec
#[async_trait(?Send)]
pub trait WavsTriggerExecClientExt: WavsBasicExecClientExt + WavsTriggerQueryClientExt {
    async fn exec(
        &self,
        msg: &mock_api::trigger::ExecuteMsg,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = self.addr();
        self.basic_contract_exec(&contract_addr, msg, funds).await
    }

    // returns the trigger ID
    async fn push_message(&self, message: impl ToString) -> Result<Uint64, cosmwasm_std::StdError> {
        let msg = mock_api::trigger::ExecuteMsg::Push {
            message: message.to_string(),
        };
        let resp = self.exec(&msg, &[]).await?;
        let events = resp.extract_events();

        let event = events
            .event_first_by_attr_key(
                PushMessageEvent::EVENT_TYPE,
                PushMessageEvent::EVENT_ATTR_KEY_TRIGGER_ID,
            )
            .map(cosmwasm_std::Event::from)
            .and_then(|e| PushMessageEvent::try_from(&e))
            .map_err(|err| cosmwasm_std::StdError::msg(err.to_string()))?;

        Ok(event.trigger_id)
    }
}
