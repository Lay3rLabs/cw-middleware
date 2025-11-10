use cosmwasm_schema::{cw_serde, QueryResponses};
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
    #[serde(untagged)]
    Wavs(ServiceManagerExecuteMessages),
}

#[cw_serde]
#[derive(QueryResponses)]
#[schemaifier(mute_warnings)]
pub enum MirrorServiceManagerQueryMessages {
    /// Get the current admin address
    #[returns(cosmwasm_std::Addr)]
    Admin {},
    /// Get the stake registry address
    #[returns(cosmwasm_std::Addr)]
    StakeRegistry {},
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
