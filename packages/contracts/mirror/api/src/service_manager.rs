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
    /// Set the admin to a new address (current admin only). Single-step
    /// because the typical post-deploy target is the
    /// mirror-quorum-sync-handler contract address. Audit C-5 / H-6 fix.
    SetAdmin { new_admin: String },
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
