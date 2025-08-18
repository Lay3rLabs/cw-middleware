use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Uint256};
use layer_climb_address::AddrEvm;

pub type InstantiateMsg = StakeRegistryInstantiateMsg;

#[cw_serde]
pub struct StakeRegistryInstantiateMsg {
    pub service_manager: String,
    pub threshold_weight: Uint256,
    pub quorum: QuorumConfig,
}

#[cw_serde]
pub struct QuorumConfig {
    pub strategies: Vec<StrategyParams>,
}

#[cw_serde]
pub struct StrategyParams {
    pub strategy: String,
    pub multiplier: Uint256,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Set operator details (owner only)
    SetOperatorDetails {
        operator: AddrEvm,
        signing_key: AddrEvm,
        weight: Uint256,
    },
    /// Batch set multiple operator details (owner only)
    BatchSetOperatorDetails {
        operators: Vec<AddrEvm>,
        signing_keys: Vec<AddrEvm>,
        weights: Vec<Uint256>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    /// Check if a signature is valid (equivalent to isValidSignature)
    /// Returns ValidationResult with total voting power and voting power that signed
    ValidateSignature {
        digest: Binary,
        signature_data: Binary,
    },
    /// Get operator weight
    GetOperatorWeight { operator: AddrEvm },
    /// Get operator signing key
    GetOperatorSigningKey { operator: AddrEvm },
    /// Get latest operator for signing key
    GetLatestOperatorForSigningKey { signing_key: AddrEvm },
    /// Get service manager address
    GetServiceManager {},
    /// Get total weight
    GetTotalWeight {},
    /// Get quorum config
    GetQuorum {},
}

#[cw_serde]
pub struct ValidationResult {
    pub is_valid: bool,
    pub total_voting_power: Uint256,
    pub voting_power_signed: Uint256,
    pub reference_block: u32,
}

#[cw_serde]
pub struct SignatureData {
    pub operators: Vec<AddrEvm>,
    pub signatures: Vec<Binary>,
    pub reference_block: u32,
}

#[cw_serde]
pub struct OperatorDetails {
    pub operator: AddrEvm,
    pub signing_key: AddrEvm,
    pub weight: Uint256,
    pub registered: bool,
}

// Events to match Solidity contract
#[cw_serde]
pub struct OperatorWeightUpdatedEvent {
    pub operator: AddrEvm,
    pub old_weight: Uint256,
    pub new_weight: Uint256,
}

impl OperatorWeightUpdatedEvent {
    pub const EVENT_TYPE: &'static str = "operator-weight-updated";
    pub const EVENT_ATTR_KEY_OPERATOR: &'static str = "operator";
    pub const EVENT_ATTR_KEY_OLD_WEIGHT: &'static str = "old-weight";
    pub const EVENT_ATTR_KEY_NEW_WEIGHT: &'static str = "new-weight";
}

impl From<OperatorWeightUpdatedEvent> for cosmwasm_std::Event {
    fn from(src: OperatorWeightUpdatedEvent) -> Self {
        cosmwasm_std::Event::new(OperatorWeightUpdatedEvent::EVENT_TYPE)
            .add_attribute(
                OperatorWeightUpdatedEvent::EVENT_ATTR_KEY_OPERATOR,
                src.operator.to_string(),
            )
            .add_attribute(
                OperatorWeightUpdatedEvent::EVENT_ATTR_KEY_OLD_WEIGHT,
                src.old_weight.to_string(),
            )
            .add_attribute(
                OperatorWeightUpdatedEvent::EVENT_ATTR_KEY_NEW_WEIGHT,
                src.new_weight.to_string(),
            )
    }
}

#[cw_serde]
pub struct TotalWeightUpdatedEvent {
    pub old_total_weight: Uint256,
    pub new_total_weight: Uint256,
}

impl TotalWeightUpdatedEvent {
    pub const EVENT_TYPE: &'static str = "total-weight-updated";
    pub const EVENT_ATTR_KEY_OLD_TOTAL_WEIGHT: &'static str = "old-total-weight";
    pub const EVENT_ATTR_KEY_NEW_TOTAL_WEIGHT: &'static str = "new-total-weight";
}

impl From<TotalWeightUpdatedEvent> for cosmwasm_std::Event {
    fn from(src: TotalWeightUpdatedEvent) -> Self {
        cosmwasm_std::Event::new(TotalWeightUpdatedEvent::EVENT_TYPE)
            .add_attribute(
                TotalWeightUpdatedEvent::EVENT_ATTR_KEY_OLD_TOTAL_WEIGHT,
                src.old_total_weight.to_string(),
            )
            .add_attribute(
                TotalWeightUpdatedEvent::EVENT_ATTR_KEY_NEW_TOTAL_WEIGHT,
                src.new_total_weight.to_string(),
            )
    }
}

#[cw_serde]
pub struct SigningKeyUpdateEvent {
    pub operator: AddrEvm,
    pub block_number: u64,
    pub new_signing_key: AddrEvm,
    pub old_signing_key: Option<AddrEvm>,
}

impl SigningKeyUpdateEvent {
    pub const EVENT_TYPE: &'static str = "signing-key-update";
    pub const EVENT_ATTR_KEY_OPERATOR: &'static str = "operator";
    pub const EVENT_ATTR_KEY_BLOCK_NUMBER: &'static str = "block-number";
    pub const EVENT_ATTR_KEY_NEW_SIGNING_KEY: &'static str = "new-signing-key";
    pub const EVENT_ATTR_KEY_OLD_SIGNING_KEY: &'static str = "old-signing-key";
}

impl From<SigningKeyUpdateEvent> for cosmwasm_std::Event {
    fn from(src: SigningKeyUpdateEvent) -> Self {
        cosmwasm_std::Event::new(SigningKeyUpdateEvent::EVENT_TYPE)
            .add_attribute(
                SigningKeyUpdateEvent::EVENT_ATTR_KEY_OPERATOR,
                src.operator.to_string(),
            )
            .add_attribute(
                SigningKeyUpdateEvent::EVENT_ATTR_KEY_BLOCK_NUMBER,
                src.block_number.to_string(),
            )
            .add_attribute(
                SigningKeyUpdateEvent::EVENT_ATTR_KEY_NEW_SIGNING_KEY,
                src.new_signing_key.to_string(),
            )
            .add_attribute(
                SigningKeyUpdateEvent::EVENT_ATTR_KEY_OLD_SIGNING_KEY,
                src.old_signing_key
                    .map(|k| k.to_string())
                    .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string()),
            )
    }
}
