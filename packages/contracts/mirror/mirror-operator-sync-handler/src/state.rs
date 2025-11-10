use cw_storage_plus::Item;

pub const LAST_TRIGGER_ID: Item<u64> = Item::new("last-trigger-id");
