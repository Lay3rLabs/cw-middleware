use alloy_primitives::{keccak256, B256};
use alloy_sol_types::SolType;
use cosmwasm_std::{
    entry_point, instantiate2_address, to_json_binary, Addr, Binary, CodeInfoResponse, Deps,
    DepsMut, Env, MessageInfo, QueryResponse, Response, StdError, StdResult, Uint256, WasmMsg,
};
use cw2::set_contract_version;
use k256::ecdsa::{RecoveryId, Signature as K256Signature, VerifyingKey};
use layer_climb_address::AddrEvm;

use crate::error::ContractError;
use crate::state::{
    Config, CONFIG, OPERATOR_REGISTERED, OPERATOR_SIGNING_KEYS, OPERATOR_WEIGHTS, OWNER,
    SIGNING_KEY_TO_OPERATOR, TOTAL_WEIGHT,
};
use cw_wavs_mirror_api::stake_registry::{
    ExecuteMsg, InstantiateMsg, OperatorWeightUpdatedEvent, QueryMsg, SignatureData,
    SigningKeyUpdateEvent, TotalWeightUpdatedEvent, ValidationResult,
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg.service_manager_instantiate {
        WasmMsg::Instantiate2 {
            code_id, ref salt, ..
        } => {
            let canonical_creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;
            let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(code_id)?;
            let service_manager =
                instantiate2_address(checksum.as_slice(), &canonical_creator, salt)?;
            let service_manager = deps.api.addr_humanize(&service_manager)?;

            let config = Config::new(service_manager, msg.threshold_weight, msg.quorum);
            CONFIG.save(deps.storage, &config)?;
        }
        _ => {
            return Err(ContractError::Std(StdError::msg(
                "Could not instantiate service manager",
            )));
        }
    }
    OWNER.save(deps.storage, &info.sender)?;
    TOTAL_WEIGHT.save(deps.storage, &Uint256::zero())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_message(msg.service_manager_instantiate))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetOperatorDetails {
            operator,
            signing_key,
            weight,
        } => execute_set_operator_details(deps, _env, info, operator, signing_key, weight),
        ExecuteMsg::BatchSetOperatorDetails {
            operators,
            signing_keys,
            weights,
        } => execute_batch_set_operator_details(deps, _env, info, operators, signing_keys, weights),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::ValidateSignature {
            digest,
            signature_data,
        } => to_json_binary(&query_validate_signature(deps, digest, signature_data)?),
        QueryMsg::GetOperatorWeight { operator } => {
            to_json_binary(&query_operator_weight(deps, operator)?)
        }
        QueryMsg::GetOperatorSigningKey { operator } => {
            to_json_binary(&query_operator_signing_key(deps, operator)?)
        }
        QueryMsg::GetLatestOperatorForSigningKey { signing_key } => {
            to_json_binary(&query_latest_operator_for_signing_key(deps, signing_key)?)
        }
        QueryMsg::GetServiceManager {} => to_json_binary(&query_service_manager(deps)?),
        QueryMsg::GetTotalWeight {} => to_json_binary(&query_total_weight(deps)?),
        QueryMsg::GetQuorum {} => to_json_binary(&query_quorum(deps)?),
    }
}

fn execute_set_operator_details(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: AddrEvm,
    signing_key: AddrEvm,
    weight: Uint256,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    let snapshot_height = env.block.height;
    let (mut events, old_total_weight, new_total_weight) =
        set_operator_details_at(deps, snapshot_height, &operator, &signing_key, weight)?;

    // Emit TotalWeightUpdated event
    let total_weight_event = TotalWeightUpdatedEvent {
        old_total_weight,
        new_total_weight,
    };
    events.push(total_weight_event.into());

    Ok(Response::new()
        .add_events(events)
        .add_attribute("method", "set_operator_details")
        .add_attribute("operator", operator.to_string())
        .add_attribute("signing_key", signing_key.to_string())
        .add_attribute("weight", weight.to_string()))
}

fn execute_batch_set_operator_details(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operators: Vec<AddrEvm>,
    signing_keys: Vec<AddrEvm>,
    weights: Vec<Uint256>,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    if operators.len() != signing_keys.len() || operators.len() != weights.len() {
        return Err(ContractError::InvalidArrayLengths {});
    }

    let mut all_events = Vec::new();
    let initial_total_weight = TOTAL_WEIGHT.load(deps.storage)?;

    for i in 0..operators.len() {
        let (events, _old_total, _new_total) = set_operator_details_at(
            deps.branch(),
            env.block.height,
            &operators[i],
            &signing_keys[i],
            weights[i],
        )?;
        all_events.extend(events);
    }

    // Emit final TotalWeightUpdated event
    let final_total_weight = TOTAL_WEIGHT.load(deps.storage)?;
    let total_weight_event = TotalWeightUpdatedEvent {
        old_total_weight: initial_total_weight,
        new_total_weight: final_total_weight,
    };
    all_events.push(total_weight_event.into());

    Ok(Response::new()
        .add_events(all_events)
        .add_attribute("method", "batch_set_operator_details")
        .add_attribute("count", operators.len().to_string()))
}

fn set_operator_details_at(
    deps: DepsMut,
    snapshot_height: u64,
    operator: &AddrEvm,
    signing_key: &AddrEvm,
    weight: Uint256,
) -> Result<(Vec<cosmwasm_std::Event>, Uint256, Uint256), ContractError> {
    // Get current weight
    let operator_key = operator.to_string();
    let current_weight = OPERATOR_WEIGHTS
        .may_load(deps.storage, operator_key.clone())?
        .unwrap_or_default();

    // Update operator weight with block height
    OPERATOR_WEIGHTS.save(deps.storage, operator_key.clone(), &weight, snapshot_height)?;

    // Update total weight
    let total_weight = TOTAL_WEIGHT.load(deps.storage)?;
    let new_total_weight = total_weight - current_weight + weight;
    TOTAL_WEIGHT.save(deps.storage, &new_total_weight)?;

    // Update signing key mappings
    let current_signing_key = OPERATOR_SIGNING_KEYS.may_load(deps.storage, operator_key.clone())?;

    let mut events = Vec::new();

    if current_signing_key.as_ref() != Some(signing_key) {
        // Remove old signing key mapping if it exists
        if let Some(old_key) = current_signing_key.clone() {
            let old_key_str = old_key.to_string();
            SIGNING_KEY_TO_OPERATOR.remove(deps.storage, old_key_str, snapshot_height)?;
        }

        // Set new signing key mapping
        let signing_key_str = signing_key.to_string();
        OPERATOR_SIGNING_KEYS.save(
            deps.storage,
            operator_key.clone(),
            signing_key,
            snapshot_height,
        )?;
        SIGNING_KEY_TO_OPERATOR.save(deps.storage, signing_key_str, operator, snapshot_height)?;

        // Emit SigningKeyUpdate event
        let signing_key_event = SigningKeyUpdateEvent {
            operator: operator.clone(),
            block_number: snapshot_height,
            new_signing_key: signing_key.clone(),
            old_signing_key: current_signing_key.clone(),
        };
        events.push(signing_key_event.into());
    }

    // Mark operator as registered
    OPERATOR_REGISTERED.save(deps.storage, operator_key, &true)?;

    // Emit OperatorWeightUpdated event
    let weight_event = OperatorWeightUpdatedEvent {
        operator: operator.clone(),
        old_weight: current_weight,
        new_weight: weight,
    };
    events.push(weight_event.into());

    Ok((events, total_weight, new_total_weight))
}

fn query_validate_signature(
    deps: Deps,
    digest: Binary,
    signature_data: Binary,
) -> StdResult<ValidationResult> {
    // Decode the signature data in the same format as Solidity
    // Expected format: abi.encode(address[] operators, bytes[] signatures, uint32 referenceBlock)
    let decoded = decode_signature_data(&signature_data)
        .map_err(|_| StdError::msg("Invalid signature data format"))?;

    let total_weight = TOTAL_WEIGHT.load(deps.storage)?;
    let mut voting_power_signed = Uint256::zero();

    // Basic sanity checks to avoid panics and invalid data
    if decoded.operators.is_empty() || decoded.operators.len() != decoded.signatures.len() {
        return Ok(ValidationResult {
            is_valid: false,
            total_voting_power: total_weight,
            voting_power_signed: Uint256::zero(),
            reference_block: decoded.reference_block,
        });
    }

    // Enforce unique signers to prevent double counting
    use std::collections::HashSet;
    let mut seen_signers: HashSet<[u8; 20]> = HashSet::new();

    // Verify each signature (operators are actually signing keys in the decoded data)
    for (i, signing_key) in decoded.operators.iter().enumerate() {
        // Reject zero address signers
        if signing_key.as_bytes().iter().all(|b| *b == 0) {
            return Ok(ValidationResult {
                is_valid: false,
                total_voting_power: total_weight,
                voting_power_signed: Uint256::zero(),
                reference_block: decoded.reference_block,
            });
        }

        // Reject duplicate signer entries
        let signer_arr: [u8; 20] = signing_key.as_bytes();
        if !seen_signers.insert(signer_arr) {
            return Ok(ValidationResult {
                is_valid: false,
                total_voting_power: total_weight,
                voting_power_signed: Uint256::zero(),
                reference_block: decoded.reference_block,
            });
        }

        // Get operator for this signing key as of reference_block (snapshot)
        let signing_key_str = signing_key.to_string();
        // Prefer snapshot at reference block; fall back to latest if no snapshot exists
        let operator = match SIGNING_KEY_TO_OPERATOR.may_load_at_height(
            deps.storage,
            signing_key_str.clone(),
            decoded.reference_block as u64,
        )? {
            Some(op) => op,
            None => SIGNING_KEY_TO_OPERATOR
                .may_load(deps.storage, signing_key_str)?
                .ok_or_else(|| StdError::msg("Signer not registered"))?,
        };

        // Determine registration at reference block by non-zero weight
        let operator_key = operator.to_string();

        // Verify signature using the signing key (safe index: len equality checked above)
        let signature = &decoded.signatures[i];
        if !is_valid_signature(&digest, signature, signing_key)? {
            return Ok(ValidationResult {
                is_valid: false,
                total_voting_power: total_weight,
                voting_power_signed: Uint256::zero(),
                reference_block: decoded.reference_block,
            });
        }

        // Add operator's weight to voting power (snapshot at reference block)
        let operator_weight_snapshot = OPERATOR_WEIGHTS.may_load_at_height(
            deps.storage,
            operator_key.clone(),
            decoded.reference_block as u64,
        )?;
        let operator_weight = operator_weight_snapshot
            .or_else(|| {
                OPERATOR_WEIGHTS
                    .may_load(deps.storage, operator_key)
                    .ok()
                    .flatten()
            })
            .unwrap_or_default();
        if operator_weight.is_zero() {
            return Ok(ValidationResult {
                is_valid: false,
                total_voting_power: total_weight,
                voting_power_signed: Uint256::zero(),
                reference_block: decoded.reference_block,
            });
        }
        voting_power_signed += operator_weight;
    }

    // Check if threshold is met
    let config = CONFIG.load(deps.storage)?;
    let is_valid = voting_power_signed >= config.threshold_weight;

    Ok(ValidationResult {
        is_valid,
        total_voting_power: total_weight,
        voting_power_signed,
        reference_block: decoded.reference_block,
    })
}

pub fn decode_signature_data(data: &Binary) -> Result<SignatureData, ContractError> {
    // Decode ABI-encoded data: (address[] operators, bytes[] signatures, uint32 referenceBlock)
    use alloy_sol_types::sol_data::*;
    type SignatureDataType = (Array<Address>, Array<Bytes>, Uint<32>);

    let (addresses, signatures_bytes, reference_block) =
        SignatureDataType::abi_decode(data.as_slice())
            .map_err(|_| ContractError::InvalidSignatureDataFormat {})?;

    // Convert addresses to AddrEvm
    let operators: Vec<AddrEvm> = addresses.into_iter().map(AddrEvm::from).collect();

    // Convert bytes to Binary
    let signatures: Vec<Binary> = signatures_bytes
        .into_iter()
        .map(|bytes| Binary::from(bytes.as_ref()))
        .collect();

    Ok(SignatureData {
        operators,
        signatures,
        reference_block,
    })
}

// Mimics Solidity's signer.isValidSignatureNow(digest, signature)
fn is_valid_signature(
    digest: &Binary,
    signature: &Binary,
    signer_address: &AddrEvm,
) -> StdResult<bool> {
    // Validate signature length (must be 65 bytes for ECDSA)
    if signature.len() != 65 {
        return Ok(false);
    }

    // Additional validation: signature must not be zero
    if signature.as_slice().iter().all(|&b| b == 0) {
        return Ok(false);
    }

    // Extract r, s, and recovery_id from signature
    let sig_bytes = signature.as_slice();
    let r_bytes: [u8; 32] = sig_bytes[0..32]
        .try_into()
        .map_err(|_| StdError::msg("Invalid signature format: r component"))?;
    let s_bytes: [u8; 32] = sig_bytes[32..64]
        .try_into()
        .map_err(|_| StdError::msg("Invalid signature format: s component"))?;
    let recovery_id = sig_bytes[64];

    // Create k256 signature from r and s
    let k256_sig = K256Signature::from_scalars(r_bytes, s_bytes)
        .map_err(|_| StdError::msg("Invalid signature scalars"))?;

    // Create recovery ID
    let recovery_id =
        RecoveryId::try_from(recovery_id).map_err(|_| StdError::msg("Invalid recovery ID"))?;

    // Convert digest to B256 for recovery
    let digest_hash =
        B256::try_from(digest.as_slice()).map_err(|_| StdError::msg("Invalid digest length"))?;

    // Recover the verifying key (public key) from signature
    let verifying_key =
        VerifyingKey::recover_from_prehash(digest_hash.as_slice(), &k256_sig, recovery_id)
            .map_err(|_| StdError::msg("Failed to recover public key"))?;

    // Convert verifying key to Ethereum address
    let public_key_bytes = verifying_key.to_encoded_point(false);
    let public_key_uncompressed = &public_key_bytes.as_bytes()[1..]; // Skip 0x04 prefix
    let addr_hash = keccak256(public_key_uncompressed);
    let recovered_address = &addr_hash[12..]; // Last 20 bytes

    // Compare with expected signer address
    Ok(recovered_address == signer_address.as_bytes())
}

fn query_operator_weight(deps: Deps, operator: AddrEvm) -> StdResult<Uint256> {
    let operator_key = operator.to_string();
    OPERATOR_WEIGHTS
        .may_load(deps.storage, operator_key)
        .map(|w| w.unwrap_or_default())
}

fn query_operator_signing_key(deps: Deps, operator: AddrEvm) -> StdResult<Option<AddrEvm>> {
    let operator_key = operator.to_string();
    OPERATOR_SIGNING_KEYS.may_load(deps.storage, operator_key)
}

fn query_latest_operator_for_signing_key(
    deps: Deps,
    signing_key: AddrEvm,
) -> StdResult<Option<AddrEvm>> {
    let signing_key_str = signing_key.to_string();
    SIGNING_KEY_TO_OPERATOR.may_load(deps.storage, signing_key_str)
}

fn query_service_manager(deps: Deps) -> StdResult<Addr> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.service_manager)
}

fn query_total_weight(deps: Deps) -> StdResult<Uint256> {
    TOTAL_WEIGHT.load(deps.storage)
}

fn query_quorum(deps: Deps) -> StdResult<cw_wavs_mirror_api::stake_registry::QuorumConfig> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.quorum)
}
