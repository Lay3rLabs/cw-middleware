use bincode::error::EncodeError;
use cosmwasm_std::{HexBinary, Uint64};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MessageWithId {
    pub trigger_id: Uint64,
    pub message: HexBinary,
}

impl MessageWithId {
    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
    }
}
