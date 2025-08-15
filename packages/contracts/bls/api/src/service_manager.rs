use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;
use wavs_types::contracts::cosmwasm::service_manager::{
    ServiceManagerExecuteMessages, ServiceManagerQueryMessages,
};

pub type InstantiateMsg = Empty;

#[cw_serde]
#[schemaifier(mute_warnings)]
pub enum ExecuteMsg {
    // TODO - uhh... bls stuff
    #[serde(untagged)]
    Wavs(ServiceManagerExecuteMessages),
}

#[cw_serde]
#[schemaifier(mute_warnings)]
pub enum QueryMsg {
    #[serde(untagged)]
    Wavs(ServiceManagerQueryMessages),
}
