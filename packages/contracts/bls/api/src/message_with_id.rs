use bincode::error::{DecodeError, EncodeError};
use cosmwasm_std::{HexBinary, Uint64};
use serde::{Deserialize, Serialize};

/// Envelope payload shape for BLS-family WAVS services. Identical to the
/// ECDSA family's; the off-chain difference is in how signatures over this
/// payload are aggregated.
#[derive(Serialize, Deserialize)]
pub struct MessageWithId {
    pub trigger_id: Uint64,
    pub message: HexBinary,
}

impl MessageWithId {
    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(bincode::serde::decode_from_slice(bytes, bincode::config::standard())?.0)
    }
}
