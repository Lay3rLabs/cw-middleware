use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{HexBinary, Uint64};
use wavs_types::contracts::cosmwasm::service_handler::{
    ServiceHandlerExecuteMessages, ServiceHandlerQueryMessages,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub service_manager: String,
}

#[cw_serde]
#[schemaifier(mute_warnings)]
pub enum ExecuteMsg {
    #[serde(untagged)]
    Wavs(ServiceHandlerExecuteMessages),
}

/// Family-specific queries for envelope persistence. After
/// `WavsHandleSignedEnvelope` validates and stores an envelope, downstream
/// callers can retrieve the persisted message + signature data by trigger id.
#[cw_serde]
#[derive(QueryResponses)]
#[schemaifier(mute_warnings)]
pub enum EcdsaServiceHandlerQueryMessages {
    /// Whether an envelope with this trigger_id has been persisted.
    #[returns(bool)]
    TriggerValidated { trigger_id: Uint64 },
    /// The persisted message bytes for the given trigger_id.
    #[returns(TriggerMessageResponse)]
    TriggerMessage { trigger_id: Uint64 },
    /// The signature data accompanying the persisted envelope for the given
    /// trigger_id.
    #[returns(wavs_types::contracts::cosmwasm::service_handler::WavsSignatureData)]
    SignatureData { trigger_id: Uint64 },
}

#[cw_serde]
#[derive(QueryResponses)]
#[query_responses(nested)]
#[schemaifier(mute_warnings)]
pub enum QueryMsg {
    #[serde(untagged)]
    Ecdsa(EcdsaServiceHandlerQueryMessages),
    #[serde(untagged)]
    Wavs(ServiceHandlerQueryMessages),
}

#[cw_serde]
pub struct TriggerMessageResponse {
    pub message: HexBinary,
}
