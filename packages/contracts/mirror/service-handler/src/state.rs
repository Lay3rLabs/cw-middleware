use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};
use cw_wavs_mirror_api::message_with_id::MessageWithId;
use wavs_types::{
    contracts::cosmwasm::service_handler::{WavsEnvelope, WavsSignatureData},
    Envelope,
};

pub const SERVICE_MANAGER: Item<Addr> = Item::new("service-manager");

pub const TRIGGER_MESSAGE: Map<Uint64, String> = Map::new("trigger-message");
pub const SIGNATURE_DATA: Map<Uint64, WavsSignatureData> = Map::new("signature-data");

pub fn save_envelope(
    storage: &mut dyn cosmwasm_std::Storage,
    envelope: WavsEnvelope,
    signature_data: WavsSignatureData,
) -> cosmwasm_std::StdResult<Envelope> {
    let envelope = envelope.decode()?;
    let message_with_id = MessageWithId::from_bytes(&envelope.payload)?;

    // Audit H-4 fix: reject duplicate trigger_id rather than silently
    // overwriting. Two different signed envelopes that happen to share a
    // trigger_id (operator-quorum signs both -- accidentally or
    // maliciously) used to silently clobber each other.
    if TRIGGER_MESSAGE.has(storage, message_with_id.trigger_id) {
        return Err(cosmwasm_std::StdError::msg(format!(
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
