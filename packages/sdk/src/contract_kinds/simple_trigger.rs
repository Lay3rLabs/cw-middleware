use cosmwasm_std::{Addr, Uint64};
use layer_climb::events::CosmosTxEvents;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use trigger_api::simple::PushMessageEvent;

use crate::client::{WavsExecutor, WavsQuerier, WavsTxResponse};

#[derive(Clone)]
pub struct SimpleTriggerQuerier {
    inner: WavsQuerier,
    pub addr: Addr,
}

impl SimpleTriggerQuerier {
    pub fn new(inner: WavsQuerier, addr: Addr) -> Self {
        Self { inner, addr }
    }
    pub async fn simple_trigger_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &trigger_api::simple::QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.inner.contract_query(&self.addr, msg).await
    }

    pub fn querier(&self) -> &WavsQuerier {
        &self.inner
    }

    pub async fn get_trigger_message(
        &self,
        trigger_id: impl Into<Uint64>,
    ) -> Result<String, cosmwasm_std::StdError> {
        self.simple_trigger_query(&trigger_api::simple::QueryMsg::TriggerMessage {
            trigger_id: trigger_id.into(),
        })
        .await
    }
}

#[derive(Clone)]
pub struct SimpleTriggerExecutor {
    inner: WavsExecutor,
    pub addr: Addr,
}

impl SimpleTriggerExecutor {
    pub fn new(inner: WavsExecutor, addr: Addr) -> Self {
        Self { inner, addr }
    }
    pub async fn simple_trigger_exec(
        &self,
        msg: &trigger_api::simple::ExecuteMsg,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.inner.contract_exec(&self.addr, msg, funds).await
    }

    pub fn executor(&self) -> &WavsExecutor {
        &self.inner
    }

    pub async fn push_message(
        &self,
        message: impl ToString,
    ) -> Result<Uint64, cosmwasm_std::StdError> {
        let msg = trigger_api::simple::ExecuteMsg::Push {
            message: message.to_string(),
        };
        let resp = self.simple_trigger_exec(&msg, &[]).await?;
        let events = CosmosTxEvents::from(&resp);

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
