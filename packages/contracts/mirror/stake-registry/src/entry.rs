use alloy_primitives::{Signature, B256};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError, StdResult, Uint256,
};
use cw2::set_contract_version;
use ethabi::{decode, ParamType, Token};
use layer_climb_address::AddrEvm;

use crate::error::ContractError;
use crate::state::{
    Config, CONFIG, OPERATOR_REGISTERED, OPERATOR_SIGNING_KEYS, OPERATOR_WEIGHTS, OWNER,
    SIGNING_KEY_TO_OPERATOR, TOTAL_WEIGHT,
};
use mirror_api::stake_registry::{
    ExecuteMsg, InstantiateMsg, OperatorWeightUpdatedEvent, QueryMsg, SignatureData,
    SigningKeyUpdateEvent, TotalWeightUpdatedEvent, ValidationResult,
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config::new(msg.service_manager, msg.threshold_weight, msg.quorum);
    CONFIG.save(deps.storage, &config)?;
    OWNER.save(deps.storage, &info.sender)?;
    TOTAL_WEIGHT.save(deps.storage, &Uint256::zero())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
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

    let (mut events, old_total_weight, new_total_weight) =
        set_operator_details(deps, &env, &operator, &signing_key, weight)?;

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
        let (events, _old_total, _new_total) = set_operator_details(
            deps.branch(),
            &env,
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

fn set_operator_details(
    deps: DepsMut,
    env: &Env,
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
    OPERATOR_WEIGHTS.save(
        deps.storage,
        operator_key.clone(),
        &weight,
        env.block.height,
    )?;

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
            SIGNING_KEY_TO_OPERATOR.remove(deps.storage, old_key_str, env.block.height)?;
        }

        // Set new signing key mapping
        let signing_key_str = signing_key.to_string();
        OPERATOR_SIGNING_KEYS.save(
            deps.storage,
            operator_key.clone(),
            signing_key,
            env.block.height,
        )?;
        SIGNING_KEY_TO_OPERATOR.save(deps.storage, signing_key_str, operator, env.block.height)?;

        // Emit SigningKeyUpdate event
        let signing_key_event = SigningKeyUpdateEvent {
            operator: operator.clone(),
            block_number: env.block.height,
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

    // Verify each signature (operators are actually signing keys in the decoded data)
    for (i, signing_key) in decoded.operators.iter().enumerate() {
        // Get operator for this signing key
        let signing_key_str = signing_key.to_string();
        let operator = SIGNING_KEY_TO_OPERATOR
            .may_load(deps.storage, signing_key_str)?
            .ok_or_else(|| StdError::msg("Signer not registered"))?;

        // Check if operator is registered
        let operator_key = operator.to_string();
        let is_registered = OPERATOR_REGISTERED
            .may_load(deps.storage, operator_key.clone())?
            .unwrap_or(false);
        if !is_registered {
            return Ok(ValidationResult {
                is_valid: false,
                total_voting_power: total_weight,
                voting_power_signed: Uint256::zero(),
                reference_block: decoded.reference_block,
            });
        }

        // Verify signature using the signing key
        let signature = &decoded.signatures[i];
        if !is_valid_signature(&digest, signature, signing_key)? {
            return Ok(ValidationResult {
                is_valid: false,
                total_voting_power: total_weight,
                voting_power_signed: Uint256::zero(),
                reference_block: decoded.reference_block,
            });
        }

        // Add operator's weight to voting power
        let operator_weight = OPERATOR_WEIGHTS
            .may_load(deps.storage, operator_key)?
            .unwrap_or_default();
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
    let param_types = vec![
        ParamType::Array(Box::new(ParamType::Address)),
        ParamType::Array(Box::new(ParamType::Bytes)),
        ParamType::Uint(32),
    ];

    let tokens = decode(&param_types, data.as_slice())
        .map_err(|_| ContractError::InvalidSignatureDataFormat {})?;

    if tokens.len() != 3 {
        return Err(ContractError::InvalidSignatureDataFormat {});
    }

    // Extract operators
    let operators = match &tokens[0] {
        Token::Array(addresses) => {
            let mut ops = Vec::new();
            for addr in addresses {
                if let Token::Address(addr_bytes) = addr {
                    // Convert the 20-byte address to AddrEvm
                    let addr_array: [u8; 20] = addr_bytes
                        .as_bytes()
                        .try_into()
                        .map_err(|_| ContractError::InvalidSignatureDataFormat {})?;
                    let addr_evm = AddrEvm::from(alloy_primitives::Address::from(addr_array));
                    ops.push(addr_evm);
                } else {
                    return Err(ContractError::InvalidSignatureDataFormat {});
                }
            }
            ops
        }
        _ => return Err(ContractError::InvalidSignatureDataFormat {}),
    };

    // Extract signatures
    let signatures = match &tokens[1] {
        Token::Array(signatures) => {
            let mut sigs = Vec::new();
            for sig in signatures {
                if let Token::Bytes(sig_bytes) = sig {
                    sigs.push(Binary::from(sig_bytes.clone()));
                } else {
                    return Err(ContractError::InvalidSignatureDataFormat {});
                }
            }
            sigs
        }
        _ => return Err(ContractError::InvalidSignatureDataFormat {}),
    };

    // Extract reference block
    let reference_block = match &tokens[2] {
        Token::Uint(block_num) => {
            let block_u64 = block_num.low_u64();
            if block_u64 > u32::MAX as u64 {
                return Err(ContractError::InvalidSignatureDataFormat {});
            }
            block_u64 as u32
        }
        _ => return Err(ContractError::InvalidSignatureDataFormat {}),
    };

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

    // Convert to format expected by alloy
    let hash = match B256::try_from(digest.as_slice()) {
        Ok(h) => h,
        Err(_) => return Ok(false),
    };

    let alloy_sig = match Signature::try_from(signature.as_slice()) {
        Ok(sig) => sig,
        Err(_) => return Ok(false),
    };

    // Perform ECDSA recovery to get the recovered address
    let recovered_addr = match alloy_sig.recover_address_from_prehash(&hash) {
        Ok(addr) => addr,
        Err(_) => return Ok(false),
    };

    // Validate that recovered address matches the expected signer address
    if recovered_addr.as_slice() != signer_address.as_bytes() {
        return Ok(false);
    }

    // Additional validation: signature must not be zero
    if signature.as_slice().iter().all(|&b| b == 0) {
        return Ok(false);
    }

    // Signature is valid
    Ok(true)
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

fn query_service_manager(deps: Deps) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.service_manager)
}

fn query_total_weight(deps: Deps) -> StdResult<Uint256> {
    TOTAL_WEIGHT.load(deps.storage)
}

fn query_quorum(deps: Deps) -> StdResult<mirror_api::stake_registry::QuorumConfig> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.quorum)
}
