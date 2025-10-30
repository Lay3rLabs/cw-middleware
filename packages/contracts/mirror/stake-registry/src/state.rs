use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint256};
use cw_storage_plus::{Item, Map, SnapshotMap, Strategy};
use cw_wavs_mirror_api::stake_registry::QuorumConfig;
use layer_climb_address::EvmAddr;

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
pub const OPERATOR_TO_SIGNING_KEY: SnapshotMap<String, EvmAddr> = SnapshotMap::new(
    "operator_to_signing_key",
    "operator_to_signing_key__checkpoints",
    "operator_to_signing_key__changelog",
    Strategy::EveryBlock,
);
pub const SIGNING_KEY_TO_OPERATOR: SnapshotMap<String, EvmAddr> = SnapshotMap::new(
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
    pub service_manager: Addr,
    pub threshold_weight: Uint256,
    pub quorum: QuorumConfig,
}

impl Config {
    pub fn new(service_manager: Addr, threshold_weight: Uint256, quorum: QuorumConfig) -> Self {
        Self {
            service_manager,
            threshold_weight,
            quorum,
        }
    }
}
