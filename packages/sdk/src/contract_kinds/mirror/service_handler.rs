use crate::{
    client::{WavsExecutor, WavsQuerier, WavsTxResponse},
    service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier},
};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(Clone)]
pub struct MirrorServiceHandlerQuerier {
    inner: ServiceHandlerQuerier,
}

impl MirrorServiceHandlerQuerier {
    pub fn new(inner: ServiceHandlerQuerier) -> Self {
        Self { inner }
    }

    pub async fn mirror_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &cw_wavs_mirror_api::service_handler::QueryMsg,
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
pub struct MirrorServiceHandlerExecutor {
    inner: ServiceHandlerExecutor,
}

impl MirrorServiceHandlerExecutor {
    pub fn new(inner: ServiceHandlerExecutor) -> Self {
        Self { inner }
    }

    pub async fn mirror_exec(
        &self,
        msg: &cw_wavs_mirror_api::service_handler::ExecuteMsg,
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
