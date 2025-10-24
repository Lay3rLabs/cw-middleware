use cosmwasm_schema::{cw_serde, QueryResponses};
#[allow(unused_imports)]
use cosmwasm_std::Addr;
use cosmwasm_std::Uint256;
use layer_climb_address::EvmAddr;
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
        operator: EvmAddr,
        signing_key: EvmAddr,
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
