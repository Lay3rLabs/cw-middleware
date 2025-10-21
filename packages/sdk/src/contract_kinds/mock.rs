use crate::{
    client::{WavsExecutor, WavsQuerier, WavsTxResponse},
    service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier},
    service_manager::{ServiceManagerExecutor, ServiceManagerQuerier},
};
use cosmwasm_std::Uint64;
use cw_wavs_mock_api::service_handler::TriggerMessageResponse;
use layer_climb::prelude::AddrEvm;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(Clone)]
pub struct MockServiceHandlerQuerier {
    inner: ServiceHandlerQuerier,
}

impl MockServiceHandlerQuerier {
    pub fn new(inner: ServiceHandlerQuerier) -> Self {
        Self { inner }
    }

    pub async fn mock_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &cw_wavs_mock_api::service_handler::QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.querier()
            .contract_query(&self.service_handler().addr, msg)
            .await
    }

    pub fn service_handler(&self) -> &ServiceHandlerQuerier {
        &self.inner
    }

    pub fn querier(&self) -> &WavsQuerier {
        self.inner.querier()
    }

    pub async fn get_handled_trigger_message(
        &self,
        trigger_id: Uint64,
    ) -> Result<String, cosmwasm_std::StdError> {
        let resp: TriggerMessageResponse = self
            .mock_query(&cw_wavs_mock_api::service_handler::QueryMsg::TriggerMessage { trigger_id })
            .await?;

        Ok(resp.message)
    }
}

#[derive(Clone)]
pub struct MockServiceHandlerExecutor {
    inner: ServiceHandlerExecutor,
}

impl MockServiceHandlerExecutor {
    pub fn new(inner: ServiceHandlerExecutor) -> Self {
        Self { inner }
    }

    pub async fn mock_exec(
        &self,
        msg: &cw_wavs_mock_api::service_handler::ExecuteMsg,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.executor()
            .contract_exec(&self.service_handler().addr, msg, funds)
            .await
    }

    pub fn service_handler(&self) -> &ServiceHandlerExecutor {
        &self.inner
    }

    pub fn executor(&self) -> &WavsExecutor {
        self.inner.executor()
    }

    pub async fn set_trigger_message(
        &self,
        trigger_id: Uint64,
        message: impl ToString,
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.mock_exec(
            &cw_wavs_mock_api::service_handler::ExecuteMsg::SetTriggerMessage {
                trigger_id,
                message: message.to_string(),
            },
            &[],
        )
        .await
    }
}

#[derive(Clone)]
pub struct MockServiceManagerQuerier {
    inner: ServiceManagerQuerier,
}

impl MockServiceManagerQuerier {
    pub fn new(inner: ServiceManagerQuerier) -> Self {
        Self { inner }
    }

    pub async fn mock_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &cw_wavs_mock_api::service_manager::QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.querier()
            .contract_query(&self.service_manager().addr, msg)
            .await
    }

    pub fn service_manager(&self) -> &ServiceManagerQuerier {
        &self.inner
    }

    pub fn querier(&self) -> &WavsQuerier {
        self.inner.querier()
    }
}

#[derive(Clone)]
pub struct MockServiceManagerExecutor {
    inner: ServiceManagerExecutor,
}

impl MockServiceManagerExecutor {
    pub fn new(inner: ServiceManagerExecutor) -> Self {
        Self { inner }
    }

    pub async fn mock_exec(
        &self,
        msg: &cw_wavs_mock_api::service_manager::ExecuteMsg,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.executor()
            .contract_exec(&self.service_manager().addr, msg, funds)
            .await
    }

    pub fn service_manager(&self) -> &ServiceManagerExecutor {
        &self.inner
    }

    pub fn executor(&self) -> &WavsExecutor {
        self.inner.executor()
    }

    pub async fn set_signing_key(
        &self,
        operator_addr: AddrEvm,
        signing_key_addr: AddrEvm,
        weight: u64,
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.mock_exec(
            &cw_wavs_mock_api::service_manager::ExecuteMsg::SetSigningKey {
                operator: operator_addr,
                signing_key: signing_key_addr,
                weight: weight.into(),
            },
            &[],
        )
        .await
    }
}
