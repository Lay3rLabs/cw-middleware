use crate::{
    client::{WavsExecutor, WavsQuerier, WavsTxResponse},
    service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier},
    service_manager::{ServiceManagerExecutor, ServiceManagerQuerier},
};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(Clone)]
pub struct BlsServiceHandlerQuerier {
    inner: ServiceHandlerQuerier,
}

impl BlsServiceHandlerQuerier {
    pub fn new(inner: ServiceHandlerQuerier) -> Self {
        Self { inner }
    }

    pub async fn bls_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &bls_api::service_handler::QueryMsg,
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
pub struct BlsServiceHandlerExecutor {
    inner: ServiceHandlerExecutor,
}

impl BlsServiceHandlerExecutor {
    pub fn new(inner: ServiceHandlerExecutor) -> Self {
        Self { inner }
    }

    pub async fn bls_exec(
        &self,
        msg: &bls_api::service_handler::ExecuteMsg,
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
pub struct BlsServiceManagerQuerier {
    inner: ServiceManagerQuerier,
}

impl BlsServiceManagerQuerier {
    pub fn new(inner: ServiceManagerQuerier) -> Self {
        Self { inner }
    }

    pub async fn bls_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &bls_api::service_manager::QueryMsg,
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
pub struct BlsServiceManagerExecutor {
    inner: ServiceManagerExecutor,
}

impl BlsServiceManagerExecutor {
    pub fn new(inner: ServiceManagerExecutor) -> Self {
        Self { inner }
    }

    pub async fn bls_exec(
        &self,
        msg: &bls_api::service_manager::ExecuteMsg,
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
