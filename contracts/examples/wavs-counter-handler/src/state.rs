use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const SERVICE_MANAGER: Item<Addr> = Item::new("service_manager");

pub const COUNTER: Item<u32> = Item::new("counter");
