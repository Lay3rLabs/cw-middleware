use bincode::error::{DecodeError, EncodeError};
use cosmwasm_std::{HexBinary, Uint64};
use serde::{Deserialize, Serialize};

/// Envelope payload shape for ECDSA-family WAVS services. Off-chain
/// components serialize this with bincode and stuff it into the envelope's
/// `payload` field. The service-handler decodes it on receipt to extract a
/// per-trigger id and persist the raw message bytes keyed by that id.
///
/// HexBinary is used (vs mirror's String) so non-utf8 payloads round-trip
/// safely.
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
