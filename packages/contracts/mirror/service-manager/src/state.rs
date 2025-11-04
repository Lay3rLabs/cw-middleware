use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const SERVICE_URI: Item<String> = Item::new("service-uri");
pub const STAKE_REGISTRY: Item<Addr> = Item::new("stake-registry");
