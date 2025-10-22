use crate::{
    client::{WavsExecutor, WavsQuerier, WavsTxResponse},
    service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier},
    service_manager::{ServiceManagerExecutor, ServiceManagerQuerier},
};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(Clone)]
pub struct EcdsaServiceHandlerQuerier {
    inner: ServiceHandlerQuerier,
}

impl EcdsaServiceHandlerQuerier {
    pub fn new(inner: ServiceHandlerQuerier) -> Self {
        Self { inner }
    }

    pub async fn ecdsa_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &cw_wavs_ecdsa_api::service_handler::QueryMsg,
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
}

#[derive(Clone)]
pub struct EcdsaServiceHandlerExecutor {
    inner: ServiceHandlerExecutor,
}

impl EcdsaServiceHandlerExecutor {
    pub fn new(inner: ServiceHandlerExecutor) -> Self {
        Self { inner }
    }

    pub async fn ecdsa_exec(
        &self,
        msg: &cw_wavs_ecdsa_api::service_handler::ExecuteMsg,
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
}

#[derive(Clone)]
pub struct EcdsaServiceManagerQuerier {
    inner: ServiceManagerQuerier,
}

impl EcdsaServiceManagerQuerier {
    pub fn new(inner: ServiceManagerQuerier) -> Self {
        Self { inner }
    }

    pub async fn ecdsa_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &cw_wavs_ecdsa_api::service_manager::QueryMsg,
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
pub struct EcdsaServiceManagerExecutor {
    inner: ServiceManagerExecutor,
}

impl EcdsaServiceManagerExecutor {
    pub fn new(inner: ServiceManagerExecutor) -> Self {
        Self { inner }
    }

    pub async fn ecdsa_exec(
        &self,
        msg: &cw_wavs_ecdsa_api::service_manager::ExecuteMsg,
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
}
