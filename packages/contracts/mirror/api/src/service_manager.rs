use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint256;
use layer_climb_address::AddrEvm;
use wavs_types::contracts::cosmwasm::service_manager::{
    ServiceManagerExecuteMessages, ServiceManagerQueryMessages,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
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
#[schemaifier(mute_warnings)]
pub enum QueryMsg {
    #[serde(untagged)]
    Wavs(ServiceManagerQueryMessages),
}
