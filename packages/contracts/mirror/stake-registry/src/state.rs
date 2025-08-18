use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint256};
use cw_storage_plus::{Item, Map, SnapshotMap, Strategy};
use layer_climb_address::AddrEvm;
use mirror_api::stake_registry::QuorumConfig;

// Contract configuration
pub const CONFIG: Item<Config> = Item::new("config");
pub const OWNER: Item<Addr> = Item::new("owner");

// Operator mappings with historical tracking
pub const OPERATOR_WEIGHTS: SnapshotMap<String, Uint256> = SnapshotMap::new(
    "operator_weights",
    "operator_weights__checkpoints",
    "operator_weights__changelog",
    Strategy::EveryBlock,
);
pub const OPERATOR_SIGNING_KEYS: SnapshotMap<String, AddrEvm> = SnapshotMap::new(
    "operator_signing_keys",
    "operator_signing_keys__checkpoints",
    "operator_signing_keys__changelog",
    Strategy::EveryBlock,
);
pub const SIGNING_KEY_TO_OPERATOR: SnapshotMap<String, AddrEvm> = SnapshotMap::new(
    "signing_key_to_operator",
    "signing_key_to_operator__checkpoints",
    "signing_key_to_operator__changelog",
    Strategy::EveryBlock,
);
pub const OPERATOR_REGISTERED: Map<String, bool> = Map::new("operator_registered");

// Weight tracking
pub const TOTAL_WEIGHT: Item<Uint256> = Item::new("total_weight");

#[cw_serde]
pub struct Config {
    pub service_manager: String,
    pub threshold_weight: Uint256,
    pub quorum: QuorumConfig,
}

impl Config {
    pub fn new(service_manager: String, threshold_weight: Uint256, quorum: QuorumConfig) -> Self {
        Self {
            service_manager,
            threshold_weight,
            quorum,
        }
    }
}
