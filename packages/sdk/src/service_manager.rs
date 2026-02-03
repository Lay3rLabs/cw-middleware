use cosmwasm_std::{Addr, StdResult};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use wavs_types::contracts::cosmwasm::{
    service_handler::{WavsEnvelope, WavsSignatureData},
    service_manager::{
        ServiceManagerExecuteMessages, ServiceManagerQueryMessages, WavsValidateResult,
    },
};

use crate::client::{WavsExecutor, WavsQuerier, WavsTxResponse};

#[derive(Clone)]
pub struct ServiceManagerQuerier {
    inner: WavsQuerier,
    pub addr: Addr,
}

impl ServiceManagerQuerier {
    pub fn new(inner: WavsQuerier, addr: Addr) -> Self {
        Self { inner, addr }
    }
    pub async fn service_manager_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &ServiceManagerQueryMessages,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.inner.contract_query(&self.addr, msg).await
    }

    pub fn querier(&self) -> &WavsQuerier {
        &self.inner
    }

    pub async fn get_service_uri(&self) -> Result<String, cosmwasm_std::StdError> {
        self.service_manager_query(&ServiceManagerQueryMessages::WavsServiceUri {})
            .await
    }

    pub async fn validate(
        &self,
        envelope: WavsEnvelope,
        signature_data: WavsSignatureData,
    ) -> StdResult<WavsValidateResult> {
        self.service_manager_query(&ServiceManagerQueryMessages::WavsValidate {
            envelope,
            signature_data,
        })
        .await
    }
}

#[derive(Clone)]
pub struct ServiceManagerExecutor {
    inner: WavsExecutor,
    pub addr: Addr,
}

impl ServiceManagerExecutor {
    pub fn new(inner: WavsExecutor, addr: Addr) -> Self {
        Self { inner, addr }
    }
    pub async fn service_manager_exec(
        &self,
        msg: &ServiceManagerExecuteMessages,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.inner.contract_exec(&self.addr, msg, funds).await
    }

    pub fn executor(&self) -> &WavsExecutor {
        &self.inner
    }

    pub async fn set_service_uri(
        &self,
        uri: String,
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        let msg = ServiceManagerExecuteMessages::WavsSetServiceUri { service_uri: uri };
        self.service_manager_exec(&msg, &[]).await
    }

    pub async fn set_quorum_threshold(
        &self,
        numerator: cosmwasm_std::Uint256,
        denominator: cosmwasm_std::Uint256,
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        let msg = ServiceManagerExecuteMessages::WavsSetQuorumThreshold {
            numerator,
            denominator,
        };
        self.service_manager_exec(&msg, &[]).await
    }
}
