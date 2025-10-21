use crate::{
    client::{WavsExecutor, WavsQuerier, WavsTxResponse},
    service_manager::{ServiceManagerExecutor, ServiceManagerQuerier},
};
use layer_climb::prelude::AddrEvm;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(Clone)]
pub struct MirrorServiceManagerQuerier {
    inner: ServiceManagerQuerier,
}

impl MirrorServiceManagerQuerier {
    pub fn new(inner: ServiceManagerQuerier) -> Self {
        Self { inner }
    }

    pub async fn mirror_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &cw_wavs_mirror_api::service_manager::QueryMsg,
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
pub struct MirrorServiceManagerExecutor {
    inner: ServiceManagerExecutor,
}

impl MirrorServiceManagerExecutor {
    pub fn new(inner: ServiceManagerExecutor) -> Self {
        Self { inner }
    }

    pub async fn mirror_exec(
        &self,
        msg: &cw_wavs_mirror_api::service_manager::ExecuteMsg,
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
        self.mirror_exec(
            &cw_wavs_mirror_api::service_manager::ExecuteMsg::SetSigningKey {
                operator: operator_addr,
                signing_key: signing_key_addr,
                weight: weight.into(),
            },
            &[],
        )
        .await
    }
}
