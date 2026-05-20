use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{HexBinary, Uint256};
use layer_climb_address::EvmAddr;
use wavs_types::contracts::cosmwasm::service_manager::{
    ServiceManagerExecuteMessages, ServiceManagerQueryMessages,
};

#[cw_serde]
pub struct InstantiateMsg {
    /// Owner key — controls operator set, BLS keys, weights, and pause.
    pub owner: String,
    /// Admin key — controls service URI and quorum threshold. Distinct from
    /// owner so an admin-key compromise can't reshape the operator set.
    pub admin: String,
    /// Optional initial quorum threshold. Defaults to 2/3 if not provided.
    pub quorum_numerator: Option<Uint256>,
    pub quorum_denominator: Option<Uint256>,
}

#[cw_serde]
#[schemaifier(mute_warnings)]
pub enum ExecuteMsg {
    /// Owner-only: register a new operator with weight and a BLS12-381 G1
    /// public key (48-byte compressed encoding per CosmWasm convention).
    /// The 20-byte signing-key id used in WavsValidate is derived as
    /// `keccak256(g1_pubkey)[..20]`.
    RegisterOperator {
        operator: String,
        bls_pubkey: HexBinary,
        weight: Uint256,
    },
    /// Owner-only: deregister an operator.
    DeregisterOperator { operator: String },
    /// Owner-only: change an operator's weight without rotating their key.
    UpdateOperatorWeight { operator: String, weight: Uint256 },
    /// Owner-only: rotate an operator's BLS pubkey. New key snapshotted at
    /// the current block; historical lookups continue to resolve the old key.
    UpdateOperatorBlsPubkey {
        operator: String,
        new_bls_pubkey: HexBinary,
    },

    /// Owner-only: freeze the contract.
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

    /// Standard WAVS service-manager messages. Both variants are admin-gated.
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
    #[returns(Option<String>)]
    PendingOwner {},
    #[returns(String)]
    Admin {},
    #[returns(Option<String>)]
    PendingAdmin {},
    #[returns(bool)]
    Paused {},

    /// Total registered weight. With `reference_block`, returns the snapshot
    /// at that height.
    #[returns(Uint256)]
    TotalWeight { reference_block: Option<u64> },
    /// An operator's weight.
    #[returns(Uint256)]
    OperatorWeight {
        operator: String,
        reference_block: Option<u64>,
    },
    /// An operator's BLS G1 pubkey (48 bytes compressed).
    #[returns(Option<HexBinary>)]
    OperatorBlsPubkey {
        operator: String,
        reference_block: Option<u64>,
    },
    /// The 20-byte signing-key id derived from the operator's BLS pubkey.
    /// This is what populates `signers` in WavsValidate calls.
    #[returns(Option<EvmAddr>)]
    OperatorSigningKeyId {
        operator: String,
        reference_block: Option<u64>,
    },
    #[returns(bool)]
    OperatorRegistered { operator: String },

    /// Standard WAVS service-manager queries. The `WavsValidate` query for
    /// the BLS family expects `signature_data.signatures.len() == 1` with
    /// the aggregate G2 signature (96 bytes compressed) in
    /// `signatures[0]`. `signers` carries 20-byte BLS-key ids derived from
    /// each contributing G1 pubkey.
    #[returns(())]
    #[serde(untagged)]
    Wavs(ServiceManagerQueryMessages),
}
