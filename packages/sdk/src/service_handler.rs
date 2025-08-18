use cosmwasm_std::Addr;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use wavs_types::contracts::cosmwasm::service_handler::{
    ServiceHandlerExecuteMessages, ServiceHandlerQueryMessages, WavsEnvelope, WavsSignatureData,
};

use crate::client::{WavsExecutor, WavsQuerier, WavsTxResponse};

#[derive(Clone)]
pub struct ServiceHandlerQuerier {
    inner: WavsQuerier,
    pub addr: Addr,
}

impl ServiceHandlerQuerier {
    pub fn new(inner: WavsQuerier, addr: Addr) -> Self {
        Self { inner, addr }
    }
    pub async fn service_handler_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &ServiceHandlerQueryMessages,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.inner.contract_query(&self.addr, msg).await
    }

    pub fn querier(&self) -> &WavsQuerier {
        &self.inner
    }

    pub async fn get_manager_address(&self) -> Result<Addr, cosmwasm_std::StdError> {
        self.service_handler_query(&ServiceHandlerQueryMessages::WavsServiceManager {})
            .await
    }
}

#[derive(Clone)]
pub struct ServiceHandlerExecutor {
    inner: WavsExecutor,
    pub addr: Addr,
}

impl ServiceHandlerExecutor {
    pub fn new(inner: WavsExecutor, addr: Addr) -> Self {
        Self { inner, addr }
    }
    pub async fn service_handler_exec(
        &self,
        msg: &ServiceHandlerExecuteMessages,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.inner.contract_exec(&self.addr, msg, funds).await
    }

    pub fn executor(&self) -> &WavsExecutor {
        &self.inner
    }

    pub async fn handle_signed_envelope(
        &self,
        envelope: WavsEnvelope,
        signature_data: WavsSignatureData,
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        let msg = ServiceHandlerExecuteMessages::WavsHandleSignedEnvelope {
            envelope,
            signature_data,
        };
        self.service_handler_exec(&msg, &[]).await
    }
}
