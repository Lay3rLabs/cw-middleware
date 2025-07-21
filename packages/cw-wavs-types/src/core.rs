use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

/// The signed envelope containing event info and service output
#[cw_serde]
#[derive(Default)]
pub struct Envelope {
    /// The event ID
    pub event_id: Binary,
    /// The ordering of the event (currently unused, for future version)
    pub ordering: Option<u64>,
    /// The output of the service execution
    pub payload: Binary,
}

/// The signature data for a signed envelope
#[cw_serde]
#[derive(Default)]
pub struct SignatureData {
    /// The signers
    pub signers: Vec<String>,
    /// The signatures per signer
    pub signatures: Vec<Binary>,
    /// The reference block
    pub reference_block: u32,
}
