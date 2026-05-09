use cosmwasm_std::{Addr, Uint256};
use cw_storage_plus::{Item, Map, SnapshotItem, SnapshotMap, Strategy};
use layer_climb_address::EvmAddr;

/// Owner — controls operator set, weights, signing keys, and pause.
pub const OWNER: Item<Addr> = Item::new("owner");
/// Pending owner during a two-step TransferOwnership.
pub const PENDING_OWNER: Item<Addr> = Item::new("pending_owner");

/// Admin — controls service URI and quorum threshold. Role separated from
/// owner so an admin-key compromise can't reshape the operator set.
pub const ADMIN: Item<Addr> = Item::new("admin");
/// Pending admin during a two-step SetAdmin.
pub const PENDING_ADMIN: Item<Addr> = Item::new("pending_admin");

/// Pause flag. Validation queries reject when true; weight-mutating
/// execute messages reject when true. Toggled by owner.
pub const PAUSED: Item<bool> = Item::new("paused");

/// Service URI. Admin-gated.
pub const SERVICE_URI: Item<String> = Item::new("service_uri");

/// Quorum threshold — snapshotted so historical envelopes can be evaluated
/// against the threshold that was in force at their reference_block (audit
/// M-4 fix; preventatively applied to the new ECDSA family).
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

/// Total weight of registered operators. Snapshot-backed for historical
/// quorum checks.
pub const TOTAL_WEIGHT: SnapshotItem<Uint256> = SnapshotItem::new(
    "total_weight",
    "total_weight__checkpoints",
    "total_weight__changelog",
    Strategy::EveryBlock,
);

/// operator (CosmWasm address as String) → weight.
pub const OPERATOR_WEIGHTS: SnapshotMap<String, Uint256> = SnapshotMap::new(
    "operator_weights",
    "operator_weights__checkpoints",
    "operator_weights__changelog",
    Strategy::EveryBlock,
);

/// operator → ETH-style signing key (secp256k1 pubkey-derived).
pub const OPERATOR_TO_SIGNING_KEY: SnapshotMap<String, EvmAddr> = SnapshotMap::new(
    "operator_to_signing_key",
    "operator_to_signing_key__checkpoints",
    "operator_to_signing_key__changelog",
    Strategy::EveryBlock,
);

/// Reverse index: signing key (as string) → operator. Snapshotted so
/// validation can authoritatively resolve historical signers.
pub const SIGNING_KEY_TO_OPERATOR: SnapshotMap<String, Addr> = SnapshotMap::new(
    "signing_key_to_operator",
    "signing_key_to_operator__checkpoints",
    "signing_key_to_operator__changelog",
    Strategy::EveryBlock,
);

/// Whether an operator is currently registered (no historical view).
pub const OPERATOR_REGISTERED: Map<String, bool> = Map::new("operator_registered");
