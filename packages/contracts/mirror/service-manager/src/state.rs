use cosmwasm_std::{Addr, Uint256};
use cw_storage_plus::{Item, Map};
use layer_climb_address::EvmAddr;

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const SERVICE_URI: Item<String> = Item::new("service-uri");
pub const STAKE_REGISTRY: Item<Addr> = Item::new("stake-registry");
pub const SIGNING_KEY_TO_OPERATOR: Map<&EvmAddr, EvmAddr> = Map::new("signing-key-to-operator");
pub const OPERATOR_WEIGHTS: Map<&EvmAddr, Uint256> = Map::new("operator-weight");
