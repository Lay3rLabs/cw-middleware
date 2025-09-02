use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint256};
use layer_climb_address::AddrEvm;
use wavs_types::contracts::cosmwasm::service_manager::ServiceManagerExecuteMessages;

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
pub enum QueryMsg {
    /// Get the current admin address
    #[returns(Addr)]
    Admin {},
    /// WAVS operator weight query
    #[returns(cosmwasm_std::Uint256)]
    WavsOperatorWeight {
        operator_address: layer_climb_address::AddrEvm,
    },
    /// WAVS signature validation
    #[returns(wavs_types::contracts::cosmwasm::service_manager::WavsValidateResult)]
    WavsValidate {
        envelope: wavs_types::contracts::cosmwasm::service_handler::WavsEnvelope,
        signature_data: wavs_types::contracts::cosmwasm::service_handler::WavsSignatureData,
    },
    /// WAVS service URI query
    #[returns(String)]
    WavsServiceUri {},
    /// WAVS latest operator for signing key
    #[returns(Option<layer_climb_address::AddrEvm>)]
    WavsLatestOperatorForSigningKey {
        signing_key_addr: layer_climb_address::AddrEvm,
    },
}
