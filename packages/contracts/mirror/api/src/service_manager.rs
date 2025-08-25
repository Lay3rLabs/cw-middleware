use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint256;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use layer_climb_address::AddrEvm;
use wavs_types::contracts::cosmwasm::service_manager::{
    ServiceManagerExecuteMessages, ServiceManagerQueryMessages,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
#[schemaifier(mute_warnings)]
pub enum ExecuteMsg {
    /// Admin-only: Set signing key for mirror service operators
    SetSigningKey {
        operator: AddrEvm,
        signing_key: AddrEvm,
        weight: Uint256,
    },
    #[serde(untagged)]
    Wavs(ServiceManagerExecuteMessages),
    #[serde(untagged)]
    Ownable(OwnableExecuteMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
#[schemaifier(mute_warnings)]
#[query_responses(nested)]
pub enum QueryMsg {
    #[serde(untagged)]
    Wavs(ServiceManagerQueryMessages),
    #[serde(untagged)]
    Ownable(OwnableQueryMsg),
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum OwnableQueryMsg {}

#[cw_ownable_execute]
#[cw_serde]
pub enum OwnableExecuteMsg {}
