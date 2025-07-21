use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::{Envelope, SignatureData};

/// The execute messages a service handler must handle
#[cw_serde]
pub enum ServiceHandlerExecuteMsg {
    /// Handles a signed envelope
    HandleSignedEnvelope {
        /// The signed envelope
        envelope: Envelope,
        /// The signature data
        signature_data: SignatureData,
    },
}

/// The query messages a service handler must handle
#[cw_serde]
#[derive(QueryResponses)]
pub enum ServiceHandlerQueryMsg {
    /// Returns the service manager address
    #[returns(String)]
    ServiceManager {},
}
