use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint256};
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
}

#[cw_serde]
#[derive(QueryResponses)]
#[schemaifier(mute_warnings)]
pub enum MirrorServiceManagerQueryMessages {
    /// Get the current admin address
    #[returns(Addr)]
    Admin {},
}

#[cw_serde]
#[derive(QueryResponses)]
#[query_responses(nested)]
#[schemaifier(mute_warnings)]
pub enum QueryMsg {
    #[serde(untagged)]
    Mirror(MirrorServiceManagerQueryMessages),
    #[serde(untagged)]
    Wavs(ServiceManagerQueryMessages),
}
