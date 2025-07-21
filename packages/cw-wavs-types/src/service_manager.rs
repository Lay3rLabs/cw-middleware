use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Deps, StdResult, Uint256};
use thiserror::Error;

use crate::{Envelope, SignatureData};

/// The execute messages a service manager must handle
#[cw_serde]
pub enum ServiceManagerExecuteMsg {
    /// Sets the service URI
    SetServiceUri {
        /// The new service URI
        service_uri: String,
    },
}

/// The query messages a service manager must handle
#[cw_serde]
#[derive(QueryResponses)]
pub enum ServiceManagerQueryMsg {
    /// Gets the operator's current weight
    #[returns(Uint256)]
    OperatorWeight {
        /// The address of the operator
        operator: String,
    },
    /// Validates a signed envelope
    #[returns(())]
    Validate {
        /// The envelope containing the data
        envelope: Envelope,
        /// The signature data
        signature_data: SignatureData,
    },
    /// Returns the service URI
    #[returns(String)]
    ServiceUri {},
    /// Returns the latest operator address associated with a signing key
    #[returns(Option<String>)]
    LatestOperatorForSigningKey {
        /// The address of the signing key
        address: String,
    },
    /// Returns the allocation manager address
    #[returns(String)]
    AllocationManager {},
    /// Returns the delegation manager address
    #[returns(String)]
    DelegationManager {},
    /// Returns the stake registry address
    #[returns(String)]
    StakeRegistry {},
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid signature data")]
    InvalidSignatureData,
    // TODO: Add actual errors
}

/// Validates the signature data for a signed envelope by querying the service manager. Errors if validation fails.
///
/// ## Arguments
///
/// * `deps` - The dependencies
/// * `service_manager` - The service manager address
/// * `envelope` - The envelope to validate
/// * `signature_data` - The signature data to validate
///
/// ## Returns
///
/// Returns `Ok(())` if validation succeeds.
///
/// ## Errors
///
/// Returns `Err(StdError)` if validation fails.
pub fn validate_signed_envelope(
    deps: &Deps,
    service_manager: &Addr,
    envelope: &Envelope,
    signature_data: SignatureData,
) -> StdResult<()> {
    deps.querier.query_wasm_smart::<()>(
        service_manager,
        &ServiceManagerQueryMsg::Validate {
            envelope: envelope.clone(),
            signature_data,
        },
    )
}
