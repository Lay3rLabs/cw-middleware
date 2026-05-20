use cosmwasm_std::{Addr, Uint256};
use cw_storage_plus::{Item, SnapshotItem, Strategy};

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const SERVICE_URI: Item<String> = Item::new("service-uri");
pub const STAKE_REGISTRY: Item<Addr> = Item::new("stake-registry");

/// Quorum configuration. Snapshot-backed so historical envelopes can be
/// evaluated against the threshold that was in force at their
/// reference_block (audit M-4 fix). Both items use `Strategy::EveryBlock`
/// to mirror the operator-set tracking in the stake-registry.
pub const QUORUM_NUMERATOR: SnapshotItem<Uint256> = SnapshotItem::new(
    "quorum_numerator",
    "quorum_numerator__checkpoints",
    "quorum_numerator__changelog",
    Strategy::EveryBlock,
);
pub const QUORUM_DENOMINATOR: SnapshotItem<Uint256> = SnapshotItem::new(
    "quorum_denominator",
    "quorum_denominator__checkpoints",
    "quorum_denominator__changelog",
    Strategy::EveryBlock,
);
