use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Registration not supported")]
    RegistrationNotSupported {},

    #[error("Deregistration not supported")]
    DeregistrationNotSupported {},

    #[error("Signing key update not supported")]
    SigningKeyUpdateNotSupported {},

    #[error("Operator update not supported")]
    OperatorUpdateNotSupported {},

    #[error("Invalid array lengths")]
    InvalidArrayLengths {},

    #[error("Invalid signature")]
    InvalidSignature {},

    #[error("Operator not registered")]
    OperatorNotRegistered {},

    #[error("Insufficient voting power")]
    InsufficientVotingPower {},

    #[error("Invalid signature data format")]
    InvalidSignatureDataFormat {},
}
