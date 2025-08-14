use cosmwasm_std::{Addr, Binary, Uint64};
use cw_storage_plus::{Item, Map};
use bls_api::data_with_id::DataWithId;
use wavs_types::contracts::cosmwasm::service_handler::{WavsEnvelope, WavsSignatureData};

pub const SERVICE_MANAGER: Item<Addr> = Item::new("service-manager");

pub const TRIGGER_DATA: Map<Uint64, Binary> = Map::new("trigger-data");
pub const SIGNATURE_DATA: Map<Uint64, WavsSignatureData> = Map::new("signature-data");

pub fn save_envelope(
    storage: &mut dyn cosmwasm_std::Storage,
    envelope: WavsEnvelope,
    signature_data: WavsSignatureData,
) -> cosmwasm_std::StdResult<()> {
    let envelope = envelope.decode()?;
    let data_with_id = DataWithId::from_bytes(&envelope.payload)?;

    TRIGGER_DATA.save(storage, data_with_id.trigger_id, &data_with_id.data)?;
    SIGNATURE_DATA.save(storage, data_with_id.trigger_id, &signature_data)?;

    Ok(())
}
