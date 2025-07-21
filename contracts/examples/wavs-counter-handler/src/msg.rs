use cosmwasm_schema::{cw_serde, QueryResponses};
use cw2::ContractVersion;
use cw_wavs_types_macros::{service_handler_execute, service_handler_query};

#[cw_serde]
pub struct InstantiateMsg {
    /// The service manager address
    pub service_manager: String,
}

#[service_handler_execute]
#[cw_serde]
pub enum ExecuteMsg {}

#[service_handler_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the counter value
    #[returns(u32)]
    Counter {},
    /// Returns the contract info
    #[returns(ContractVersion)]
    Info {},
}
