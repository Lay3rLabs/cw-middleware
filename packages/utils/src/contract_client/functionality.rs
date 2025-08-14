pub mod bls;
pub mod ecdsa;
pub mod mock;
pub mod trigger;

pub use bls::*;
pub use ecdsa::*;
pub use mock::*;
use mock_api::trigger::PushMessageEvent;
#[allow(unused_imports)]
pub use trigger::*;

use super::ext::*;
use async_trait::async_trait;
use cosmwasm_std::{Addr, Coin, Uint64};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use wavs_types::contracts::cosmwasm::{
    service_handler::{
        ServiceHandlerExecuteMessages, ServiceHandlerQueryMessages, WavsEnvelope, WavsSignatureData,
    },
    service_manager::{ServiceManagerExecuteMessages, ServiceManagerQueryMessages},
};

// This is where we define the actual functionality for the clients
// it will automatically exist for every client that implements the stuff in `ext.rs`

// Service Handler Query
#[async_trait(?Send)]
pub trait WavsServiceHandlerQueryClientExt:
    WavsBasicQueryClientExt + WavsServiceHandlerAddrExt
{
    async fn query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &ServiceHandlerQueryMessages,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = self.addr();
        self.basic_contract_query(&contract_addr, msg).await
    }

    async fn get_manager_address(&self) -> Result<Addr, cosmwasm_std::StdError> {
        self.query(&ServiceHandlerQueryMessages::WavsServiceManager {})
            .await
    }
}

// Service Handler Exec
#[async_trait(?Send)]
pub trait WavsServiceHandlerExecClientExt:
    WavsBasicExecClientExt + WavsServiceHandlerQueryClientExt
{
    async fn exec(
        &self,
        msg: &ServiceHandlerExecuteMessages,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = self.addr();
        self.basic_contract_exec(&contract_addr, msg, funds).await
    }

    async fn handle_signed_envelope(
        &self,
        envelope: WavsEnvelope,
        signature_data: WavsSignatureData,
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let msg = ServiceHandlerExecuteMessages::WavsHandleSignedEnvelope {
            envelope,
            signature_data,
        };
        self.exec(&msg, &[]).await
    }
}

// Service Manager Query
#[async_trait(?Send)]
pub trait WavsServiceManagerQueryClientExt:
    WavsBasicQueryClientExt + WavsServiceManagerAddrExt
{
    async fn query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &ServiceManagerQueryMessages,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = self.addr();
        self.basic_contract_query(&contract_addr, msg).await
    }

    async fn get_service_uri(&self) -> Result<String, cosmwasm_std::StdError> {
        self.query(&ServiceManagerQueryMessages::WavsServiceUri {})
            .await
    }
}

// Service Manager Exec
#[async_trait(?Send)]
pub trait WavsServiceManagerExecClientExt:
    WavsBasicExecClientExt + WavsServiceManagerQueryClientExt
{
    async fn exec(
        &self,
        msg: &ServiceManagerExecuteMessages,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = self.addr();
        self.basic_contract_exec(&contract_addr, msg, funds).await
    }

    async fn set_service_uri(
        &self,
        uri: String,
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let msg = ServiceManagerExecuteMessages::WavsSetServiceUri { service_uri: uri };
        self.exec(&msg, &[]).await
    }
}

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
