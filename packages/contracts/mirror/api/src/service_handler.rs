use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint64;
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

#[cw_serde]
#[derive(QueryResponses)]
#[schemaifier(mute_warnings)]
pub enum MirrorServiceHandlerQueryMessages {
    #[returns(bool)]
    TriggerValidated { trigger_id: Uint64 },

    #[returns(TriggerMessageResponse)]
    TriggerMessage { trigger_id: Uint64 },

    #[returns(wavs_types::contracts::cosmwasm::service_handler::WavsSignatureData)]
    SignatureData { trigger_id: Uint64 },
}

#[cw_serde]
#[derive(QueryResponses)]
#[query_responses(nested)]
#[schemaifier(mute_warnings)]
pub enum QueryMsg {
    #[serde(untagged)]
    Mirror(MirrorServiceHandlerQueryMessages),
    #[serde(untagged)]
    Wavs(ServiceHandlerQueryMessages),
}

#[cw_serde]
pub struct TriggerMessageResponse {
    pub message: String,
}
