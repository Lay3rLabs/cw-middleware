use cosmwasm_std::{Addr, HexBinary, Uint64};
use cw_storage_plus::{Item, Map};

pub const TRIGGER_MESSAGES: Map<Uint64, HexBinary> = Map::new("trigger-messages");
pub const TRIGGER_MESSAGE_COUNT: Item<u64> = Item::new("trigger-message-count");

/// Optional pusher allowlist. None = public bus; Some([..]) = only listed
/// addresses may Push (audit M-1).
pub const ALLOWED_PUSHERS: Item<Option<Vec<Addr>>> = Item::new("allowed-pushers");
