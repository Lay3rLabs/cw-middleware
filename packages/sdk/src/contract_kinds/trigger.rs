use cosmwasm_std::{Addr, HexBinary, Uint64};
use cw_wavs_trigger_api::simple::PushMessageEvent;
use layer_climb::events::CosmosTxEvents;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

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
    pub async fn cw_wavs_trigger_simple_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &cw_wavs_trigger_api::simple::QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.inner.contract_query(&self.addr, msg).await
    }

    pub fn querier(&self) -> &WavsQuerier {
        &self.inner
    }

    pub async fn get_trigger_message(
        &self,
        trigger_id: impl Into<Uint64>,
    ) -> Result<HexBinary, cosmwasm_std::StdError> {
        self.cw_wavs_trigger_simple_query(&cw_wavs_trigger_api::simple::QueryMsg::TriggerMessage {
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
    pub async fn cw_wavs_trigger_simple_exec(
        &self,
        msg: &cw_wavs_trigger_api::simple::ExecuteMsg,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.inner.contract_exec(&self.addr, msg, funds).await
    }

    pub fn executor(&self) -> &WavsExecutor {
        &self.inner
    }

    pub async fn push_message(&self, message: Vec<u8>) -> Result<Uint64, cosmwasm_std::StdError> {
        let msg = cw_wavs_trigger_api::simple::ExecuteMsg::Push {
            data: message.into(),
        };
        let resp = self.cw_wavs_trigger_simple_exec(&msg, &[]).await?;
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
