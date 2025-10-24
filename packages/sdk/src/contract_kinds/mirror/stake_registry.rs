use crate::client::{WavsExecutor, WavsQuerier, WavsTxResponse};
use cosmwasm_std::{Addr, Binary, Uint256};
use cw_wavs_mirror_api::stake_registry::{ExecuteMsg, QueryMsg, ValidationResult};
use layer_climb::prelude::EvmAddr;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(Clone)]
pub struct MirrorStakeRegistryQuerier {
    inner: WavsQuerier,
    pub addr: Addr,
}

impl MirrorStakeRegistryQuerier {
    pub fn new(inner: WavsQuerier, addr: Addr) -> Self {
        Self { inner, addr }
    }

    pub async fn cw_wavs_mirror_stake_registry_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
    >(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.querier().contract_query(&self.addr, msg).await
    }

    pub fn querier(&self) -> &WavsQuerier {
        &self.inner
    }

    pub async fn validate_signature(
        &self,
        digest: Binary,
        signature_data: Binary,
    ) -> Result<ValidationResult, cosmwasm_std::StdError> {
        self.cw_wavs_mirror_stake_registry_query(&QueryMsg::ValidateSignature {
            digest,
            signature_data,
        })
        .await
    }

    pub async fn get_operator_weight(
        &self,
        operator: EvmAddr,
    ) -> Result<Uint256, cosmwasm_std::StdError> {
        self.cw_wavs_mirror_stake_registry_query(&QueryMsg::GetOperatorWeight { operator })
            .await
    }

    pub async fn get_operator_signing_key(
        &self,
        operator: EvmAddr,
    ) -> Result<Option<EvmAddr>, cosmwasm_std::StdError> {
        self.cw_wavs_mirror_stake_registry_query(&QueryMsg::GetOperatorSigningKey { operator })
            .await
    }

    pub async fn get_latest_operator_for_signing_key(
        &self,
        signing_key: EvmAddr,
    ) -> Result<Option<EvmAddr>, cosmwasm_std::StdError> {
        self.cw_wavs_mirror_stake_registry_query(&QueryMsg::GetLatestOperatorForSigningKey {
            signing_key,
        })
        .await
    }

    pub async fn get_service_manager(&self) -> Result<String, cosmwasm_std::StdError> {
        self.cw_wavs_mirror_stake_registry_query(&QueryMsg::GetServiceManager {})
            .await
    }

    pub async fn get_total_weight(&self) -> Result<Uint256, cosmwasm_std::StdError> {
        self.cw_wavs_mirror_stake_registry_query(&QueryMsg::GetTotalWeight {})
            .await
    }
}

#[derive(Clone)]
pub struct MirrorStakeRegistryExecutor {
    inner: WavsExecutor,
    pub addr: Addr,
}

impl MirrorStakeRegistryExecutor {
    pub fn new(inner: WavsExecutor, addr: Addr) -> Self {
        Self { inner, addr }
    }

    pub async fn mirror_exec(
        &self,
        msg: &ExecuteMsg,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.executor().contract_exec(&self.addr, msg, funds).await
    }

    pub fn executor(&self) -> &WavsExecutor {
        &self.inner
    }

    pub async fn set_operator_details(
        &self,
        operator: EvmAddr,
        signing_key: EvmAddr,
        weight: Uint256,
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.mirror_exec(
            &ExecuteMsg::SetOperatorDetails {
                operator,
                signing_key,
                weight,
            },
            &[],
        )
        .await
    }

    pub async fn batch_set_operator_details(
        &self,
        operators: Vec<EvmAddr>,
        signing_keys: Vec<EvmAddr>,
        weights: Vec<Uint256>,
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.mirror_exec(
            &ExecuteMsg::BatchSetOperatorDetails {
                operators,
                signing_keys,
                weights,
            },
            &[],
        )
        .await
    }
}
