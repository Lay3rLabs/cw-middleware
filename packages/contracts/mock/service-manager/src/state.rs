use cosmwasm_std::Uint256;
use cw_storage_plus::{Item, Map};
use layer_climb_address::EvmAddr;

pub const SERVICE_URI: Item<String> = Item::new("service-uri");
// Lookup from operators to signer addrs
pub const OPERATOR_SIGNING_KEY_ADDRS: Map<&EvmAddr, EvmAddr> =
    Map::new("operator-signing-key-addrs");
// Reverse lookup from signer to operator addrs
pub const SIGNING_KEY_OPERATOR_ADDRS: Map<&EvmAddr, EvmAddr> =
    Map::new("signing-key-operator-addrs");
pub const OPERATOR_WEIGHTS: Map<&EvmAddr, Uint256> = Map::new("operator-weight");

// Quorum configuration
pub const QUORUM_NUMERATOR: Item<Uint256> = Item::new("quorum_numerator");
pub const QUORUM_DENOMINATOR: Item<Uint256> = Item::new("quorum_denominator");
