use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint256;
use layer_climb_address::EvmAddr;
use wavs_types::contracts::cosmwasm::service_manager::{
    ServiceManagerExecuteMessages, ServiceManagerQueryMessages,
};

#[cw_serde]
pub struct InstantiateMsg {
    /// Owner key — controls operator set, weights, signing keys, and pause.
    pub owner: String,
    /// Admin key — controls service URI and quorum threshold. Distinct from
    /// owner so an admin-key compromise cannot reshape the operator set.
    pub admin: String,
    /// Optional initial quorum threshold. Defaults to 2/3 if not provided.
    pub quorum_numerator: Option<Uint256>,
    pub quorum_denominator: Option<Uint256>,
}

#[cw_serde]
#[schemaifier(mute_warnings)]
pub enum ExecuteMsg {
    /// Owner-only: register a new operator with weight and a secp256k1 signing
    /// key (Ethereum-style address derived from the operator's pubkey).
    RegisterOperator {
        operator: String,
        signing_key: EvmAddr,
        weight: Uint256,
    },
    /// Owner-only: deregister an operator. Weight removed at current block;
    /// historical snapshots preserved.
    DeregisterOperator { operator: String },
    /// Owner-only: change an operator's weight without touching their signing
    /// key. Total weight recomputed via checked arithmetic.
    UpdateOperatorWeight { operator: String, weight: Uint256 },
    /// Owner-only: rotate an operator's signing key. New key snapshotted at
    /// the current block; historical lookups continue to resolve the old key.
    UpdateOperatorSigningKey {
        operator: String,
        new_signing_key: EvmAddr,
    },

    /// Owner-only: freeze the contract. Validation queries return `Paused`;
    /// weight-mutating writes reject.
    Pause {},
    /// Owner-only: resume normal operation.
    Unpause {},

    /// Owner-only: begin a two-step ownership transfer.
    TransferOwnership { new_owner: String },
    /// Pending-owner-only: complete the two-step ownership transfer.
    AcceptOwnership {},
    /// Admin-only: begin a two-step admin transfer.
    SetAdmin { new_admin: String },
    /// Pending-admin-only: complete the two-step admin transfer.
    AcceptAdmin {},

    /// Standard WAVS service-manager messages. Untagged by upstream protocol
    /// convention (see `wavs_types::contracts::cosmwasm::service_manager`).
    /// Both variants are admin-gated.
    #[serde(untagged)]
    Wavs(ServiceManagerExecuteMessages),
}

#[cw_serde]
#[derive(QueryResponses)]
#[schemaifier(mute_warnings)]
pub enum QueryMsg {
    /// Current owner.
    #[returns(String)]
    Owner {},
    /// Pending owner, if a transfer is in flight.
    #[returns(Option<String>)]
    PendingOwner {},
    /// Current admin.
    #[returns(String)]
    Admin {},
    /// Pending admin, if a transfer is in flight.
    #[returns(Option<String>)]
    PendingAdmin {},
    /// Whether the contract is paused.
    #[returns(bool)]
    Paused {},

    /// Total registered weight. With `reference_block`, returns the snapshot
    /// at that height.
    #[returns(Uint256)]
    TotalWeight { reference_block: Option<u64> },
    /// An operator's weight. With `reference_block`, returns the snapshot at
    /// that height.
    #[returns(Uint256)]
    OperatorWeight {
        operator: String,
        reference_block: Option<u64>,
    },
    /// An operator's signing key. With `reference_block`, returns the
    /// snapshot at that height.
    #[returns(Option<EvmAddr>)]
    OperatorSigningKey {
        operator: String,
        reference_block: Option<u64>,
    },
    /// Whether an operator is currently registered.
    #[returns(bool)]
    OperatorRegistered { operator: String },

    /// Standard WAVS service-manager queries. Untagged per upstream
    /// convention.
    #[returns(())]
    #[serde(untagged)]
    Wavs(ServiceManagerQueryMessages),
}
