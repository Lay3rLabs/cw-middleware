use cosmwasm_std::{Addr, HexBinary, Uint256};
use cw_storage_plus::{Item, Map, SnapshotItem, SnapshotMap, Strategy};
use layer_climb_address::EvmAddr;

pub const OWNER: Item<Addr> = Item::new("owner");
pub const PENDING_OWNER: Item<Addr> = Item::new("pending_owner");

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const PENDING_ADMIN: Item<Addr> = Item::new("pending_admin");

pub const PAUSED: Item<bool> = Item::new("paused");

pub const SERVICE_URI: Item<String> = Item::new("service_uri");

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

pub const TOTAL_WEIGHT: SnapshotItem<Uint256> = SnapshotItem::new(
    "total_weight",
    "total_weight__checkpoints",
    "total_weight__changelog",
    Strategy::EveryBlock,
);

/// operator → weight.
pub const OPERATOR_WEIGHTS: SnapshotMap<String, Uint256> = SnapshotMap::new(
    "operator_weights",
    "operator_weights__checkpoints",
    "operator_weights__changelog",
    Strategy::EveryBlock,
);

/// operator → 48-byte BLS12-381 G1 compressed pubkey.
pub const OPERATOR_TO_BLS_PUBKEY: SnapshotMap<String, HexBinary> = SnapshotMap::new(
    "operator_to_bls_pubkey",
    "operator_to_bls_pubkey__checkpoints",
    "operator_to_bls_pubkey__changelog",
    Strategy::EveryBlock,
);

/// 20-byte BLS-key id (= keccak256(g1_pubkey)[..20]) → operator. The id is
/// what populates `signers` in WavsSignatureData since the upstream
/// wavs-types schema constrains signers to `Vec<EvmAddr>` (20 bytes).
pub const BLS_KEY_ID_TO_OPERATOR: SnapshotMap<String, Addr> = SnapshotMap::new(
    "bls_key_id_to_operator",
    "bls_key_id_to_operator__checkpoints",
    "bls_key_id_to_operator__changelog",
    Strategy::EveryBlock,
);

/// operator → 20-byte BLS-key id, mirror of OPERATOR_TO_BLS_PUBKEY's derived
/// id (so we can answer `OperatorSigningKeyId` cheaply).
pub const OPERATOR_TO_BLS_KEY_ID: SnapshotMap<String, EvmAddr> = SnapshotMap::new(
    "operator_to_bls_key_id",
    "operator_to_bls_key_id__checkpoints",
    "operator_to_bls_key_id__changelog",
    Strategy::EveryBlock,
);

pub const OPERATOR_REGISTERED: Map<String, bool> = Map::new("operator_registered");
