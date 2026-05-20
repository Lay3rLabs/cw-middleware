use cosmwasm_std::{Addr, HexBinary, StdError, StdResult, Storage, Uint64};
use cw_storage_plus::{Item, Map};
use cw_wavs_ecdsa_api::message_with_id::MessageWithId;
use wavs_types::{
    contracts::cosmwasm::service_handler::{WavsEnvelope, WavsSignatureData},
    Envelope,
};

pub const SERVICE_MANAGER: Item<Addr> = Item::new("service-manager");

pub const TRIGGER_MESSAGE: Map<Uint64, HexBinary> = Map::new("trigger-message");
pub const SIGNATURE_DATA: Map<Uint64, WavsSignatureData> = Map::new("signature-data");

/// Persist a validated envelope, keyed by the trigger_id pulled from its
/// MessageWithId payload. Rejects duplicate trigger_ids so a second envelope
/// for the same trigger cannot silently overwrite the first (audit H-4
/// preventatively applied to the new ECDSA family).
pub fn save_envelope(
    storage: &mut dyn Storage,
    envelope: WavsEnvelope,
    signature_data: WavsSignatureData,
) -> StdResult<Envelope> {
    let envelope = envelope.decode()?;
    let message_with_id = MessageWithId::from_bytes(&envelope.payload)
        .map_err(|e| StdError::msg(format!("invalid envelope payload: {e}")))?;

    if TRIGGER_MESSAGE.has(storage, message_with_id.trigger_id) {
        return Err(StdError::msg(format!(
            "trigger_id {} already persisted",
            message_with_id.trigger_id
        )));
    }

    TRIGGER_MESSAGE.save(
        storage,
        message_with_id.trigger_id,
        &message_with_id.message,
    )?;
    SIGNATURE_DATA.save(storage, message_with_id.trigger_id, &signature_data)?;

    Ok(envelope)
}
