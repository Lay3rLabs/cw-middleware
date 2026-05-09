use std::collections::HashSet;

use alloy_primitives::{eip191_hash_message, keccak256};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Deps, DepsMut, Env, Event, HexBinary, MessageInfo,
    QueryResponse, Response, StdError, StdResult, Uint256, Uint512,
};
use cw2::set_contract_version;
use layer_climb_address::EvmAddr;
use sha3::{Digest, Keccak256};
use wavs_types::contracts::cosmwasm::{
    service_handler::{WavsEnvelope, WavsSignatureData},
    service_manager::{
        error::WavsValidateError, event::WavsServiceUriUpdatedEvent, ServiceManagerExecuteMessages,
        ServiceManagerQueryMessages, WavsValidateResult,
    },
};

use crate::state::{
    ADMIN, OPERATOR_REGISTERED, OPERATOR_TO_SIGNING_KEY, OPERATOR_WEIGHTS, OWNER, PAUSED,
    PENDING_ADMIN, PENDING_OWNER, QUORUM_DENOMINATOR, QUORUM_NUMERATOR, SERVICE_URI,
    SIGNING_KEY_TO_OPERATOR, TOTAL_WEIGHT,
};
use cw_wavs_ecdsa_api::service_manager::{ExecuteMsg, InstantiateMsg, QueryMsg};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const SECP256K1_SIGNATURE_LEN: usize = 65;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    let admin = deps.api.addr_validate(&msg.admin)?;

    OWNER.save(deps.storage, &owner)?;
    ADMIN.save(deps.storage, &admin)?;
    PAUSED.save(deps.storage, &false)?;

    let numerator = msg.quorum_numerator.unwrap_or(Uint256::from(2u128));
    let denominator = msg.quorum_denominator.unwrap_or(Uint256::from(3u128));
    validate_quorum_params(numerator, denominator)?;
    QUORUM_NUMERATOR.save(deps.storage, &numerator, env.block.height)?;
    QUORUM_DENOMINATOR.save(deps.storage, &denominator, env.block.height)?;

    TOTAL_WEIGHT.save(deps.storage, &Uint256::zero(), env.block.height)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("admin", admin)
        .add_attribute("quorum_numerator", numerator.to_string())
        .add_attribute("quorum_denominator", denominator.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::RegisterOperator {
            operator,
            signing_key,
            weight,
        } => execute_register_operator(deps, env, info, operator, signing_key, weight),
        ExecuteMsg::DeregisterOperator { operator } => {
            execute_deregister_operator(deps, env, info, operator)
        }
        ExecuteMsg::UpdateOperatorWeight { operator, weight } => {
            execute_update_operator_weight(deps, env, info, operator, weight)
        }
        ExecuteMsg::UpdateOperatorSigningKey {
            operator,
            new_signing_key,
        } => execute_update_operator_signing_key(deps, env, info, operator, new_signing_key),

        ExecuteMsg::Pause {} => execute_set_paused(deps, info, true),
        ExecuteMsg::Unpause {} => execute_set_paused(deps, info, false),

        ExecuteMsg::TransferOwnership { new_owner } => {
            execute_transfer_ownership(deps, info, new_owner)
        }
        ExecuteMsg::AcceptOwnership {} => execute_accept_ownership(deps, info),
        ExecuteMsg::SetAdmin { new_admin } => execute_set_admin(deps, info, new_admin),
        ExecuteMsg::AcceptAdmin {} => execute_accept_admin(deps, info),

        ExecuteMsg::Wavs(msg) => match msg {
            ServiceManagerExecuteMessages::WavsSetQuorumThreshold {
                numerator,
                denominator,
            } => execute_set_quorum_threshold(deps, env, info, numerator, denominator),
            ServiceManagerExecuteMessages::WavsSetServiceUri { service_uri } => {
                execute_set_service_uri(deps, info, service_uri)
            }
        },
    }
}

fn ensure_owner(deps: Deps, info: &MessageInfo) -> StdResult<Addr> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(StdError::msg("Unauthorized: owner only"));
    }
    Ok(owner)
}

fn ensure_admin(deps: Deps, info: &MessageInfo) -> StdResult<Addr> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(StdError::msg("Unauthorized: admin only"));
    }
    Ok(admin)
}

fn ensure_not_paused(deps: Deps) -> StdResult<()> {
    if PAUSED.load(deps.storage)? {
        return Err(StdError::msg("Contract is paused"));
    }
    Ok(())
}

fn execute_register_operator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    signing_key: EvmAddr,
    weight: Uint256,
) -> StdResult<Response> {
    ensure_owner(deps.as_ref(), &info)?;
    ensure_not_paused(deps.as_ref())?;

    let operator_addr = deps.api.addr_validate(&operator)?;
    let operator_key = operator_addr.to_string();

    if OPERATOR_REGISTERED
        .may_load(deps.storage, operator_key.clone())?
        .unwrap_or(false)
    {
        return Err(StdError::msg("operator already registered"));
    }

    if signing_key_is_zero(&signing_key) {
        return Err(StdError::msg("signing key is zero"));
    }

    let signing_key_str = signing_key.to_string();
    if SIGNING_KEY_TO_OPERATOR
        .may_load(deps.storage, signing_key_str.clone())?
        .is_some()
    {
        return Err(StdError::msg("signing key already in use"));
    }

    let snapshot_height = env.block.height;

    OPERATOR_REGISTERED.save(deps.storage, operator_key.clone(), &true)?;
    OPERATOR_WEIGHTS.save(deps.storage, operator_key.clone(), &weight, snapshot_height)?;
    OPERATOR_TO_SIGNING_KEY.save(
        deps.storage,
        operator_key.clone(),
        &signing_key,
        snapshot_height,
    )?;
    SIGNING_KEY_TO_OPERATOR.save(
        deps.storage,
        signing_key_str.clone(),
        &operator_addr,
        snapshot_height,
    )?;

    let old_total = TOTAL_WEIGHT.load(deps.storage)?;
    let new_total = old_total
        .checked_add(weight)
        .map_err(|e| StdError::msg(format!("total weight overflow: {e}")))?;
    TOTAL_WEIGHT.save(deps.storage, &new_total, snapshot_height)?;

    Ok(Response::new()
        .add_attribute("method", "register_operator")
        .add_attribute("operator", operator_key)
        .add_attribute("signing_key", signing_key_str)
        .add_attribute("weight", weight.to_string())
        .add_attribute("total_weight", new_total.to_string()))
}

fn execute_deregister_operator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
) -> StdResult<Response> {
    ensure_owner(deps.as_ref(), &info)?;
    ensure_not_paused(deps.as_ref())?;

    let operator_addr = deps.api.addr_validate(&operator)?;
    let operator_key = operator_addr.to_string();

    if !OPERATOR_REGISTERED
        .may_load(deps.storage, operator_key.clone())?
        .unwrap_or(false)
    {
        return Err(StdError::msg("operator not registered"));
    }

    let snapshot_height = env.block.height;
    let current_weight = OPERATOR_WEIGHTS
        .may_load(deps.storage, operator_key.clone())?
        .unwrap_or_default();
    let current_signing_key = OPERATOR_TO_SIGNING_KEY
        .may_load(deps.storage, operator_key.clone())?
        .ok_or_else(|| StdError::msg("operator has no signing key"))?;

    OPERATOR_WEIGHTS.remove(deps.storage, operator_key.clone(), snapshot_height)?;
    OPERATOR_TO_SIGNING_KEY.remove(deps.storage, operator_key.clone(), snapshot_height)?;
    SIGNING_KEY_TO_OPERATOR.remove(
        deps.storage,
        current_signing_key.to_string(),
        snapshot_height,
    )?;
    OPERATOR_REGISTERED.remove(deps.storage, operator_key.clone());

    let old_total = TOTAL_WEIGHT.load(deps.storage)?;
    let new_total = old_total
        .checked_sub(current_weight)
        .map_err(|e| StdError::msg(format!("total weight underflow: {e}")))?;
    TOTAL_WEIGHT.save(deps.storage, &new_total, snapshot_height)?;

    Ok(Response::new()
        .add_attribute("method", "deregister_operator")
        .add_attribute("operator", operator_key)
        .add_attribute("removed_weight", current_weight.to_string())
        .add_attribute("total_weight", new_total.to_string()))
}

fn execute_update_operator_weight(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    weight: Uint256,
) -> StdResult<Response> {
    ensure_owner(deps.as_ref(), &info)?;
    ensure_not_paused(deps.as_ref())?;

    let operator_addr = deps.api.addr_validate(&operator)?;
    let operator_key = operator_addr.to_string();

    if !OPERATOR_REGISTERED
        .may_load(deps.storage, operator_key.clone())?
        .unwrap_or(false)
    {
        return Err(StdError::msg("operator not registered"));
    }

    let snapshot_height = env.block.height;
    let current_weight = OPERATOR_WEIGHTS
        .may_load(deps.storage, operator_key.clone())?
        .unwrap_or_default();

    OPERATOR_WEIGHTS.save(deps.storage, operator_key.clone(), &weight, snapshot_height)?;

    // Recompute total weight via checked ops (audit H-3 preventative).
    let old_total = TOTAL_WEIGHT.load(deps.storage)?;
    let new_total = old_total
        .checked_sub(current_weight)
        .map_err(|e| StdError::msg(format!("total weight underflow: {e}")))?
        .checked_add(weight)
        .map_err(|e| StdError::msg(format!("total weight overflow: {e}")))?;
    TOTAL_WEIGHT.save(deps.storage, &new_total, snapshot_height)?;

    Ok(Response::new()
        .add_attribute("method", "update_operator_weight")
        .add_attribute("operator", operator_key)
        .add_attribute("old_weight", current_weight.to_string())
        .add_attribute("new_weight", weight.to_string())
        .add_attribute("total_weight", new_total.to_string()))
}

fn execute_update_operator_signing_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    new_signing_key: EvmAddr,
) -> StdResult<Response> {
    ensure_owner(deps.as_ref(), &info)?;
    ensure_not_paused(deps.as_ref())?;

    let operator_addr = deps.api.addr_validate(&operator)?;
    let operator_key = operator_addr.to_string();

    if !OPERATOR_REGISTERED
        .may_load(deps.storage, operator_key.clone())?
        .unwrap_or(false)
    {
        return Err(StdError::msg("operator not registered"));
    }

    if signing_key_is_zero(&new_signing_key) {
        return Err(StdError::msg("signing key is zero"));
    }

    let new_signing_key_str = new_signing_key.to_string();
    if let Some(existing) =
        SIGNING_KEY_TO_OPERATOR.may_load(deps.storage, new_signing_key_str.clone())?
    {
        if existing != operator_addr {
            return Err(StdError::msg("signing key already in use"));
        }
    }

    let snapshot_height = env.block.height;
    let old_signing_key = OPERATOR_TO_SIGNING_KEY
        .may_load(deps.storage, operator_key.clone())?
        .ok_or_else(|| StdError::msg("operator has no signing key"))?;

    if old_signing_key == new_signing_key {
        return Err(StdError::msg("signing key unchanged"));
    }

    SIGNING_KEY_TO_OPERATOR.remove(
        deps.storage,
        old_signing_key.to_string(),
        snapshot_height,
    )?;
    OPERATOR_TO_SIGNING_KEY.save(
        deps.storage,
        operator_key.clone(),
        &new_signing_key,
        snapshot_height,
    )?;
    SIGNING_KEY_TO_OPERATOR.save(
        deps.storage,
        new_signing_key_str.clone(),
        &operator_addr,
        snapshot_height,
    )?;

    Ok(Response::new()
        .add_attribute("method", "update_operator_signing_key")
        .add_attribute("operator", operator_key)
        .add_attribute("old_signing_key", old_signing_key.to_string())
        .add_attribute("new_signing_key", new_signing_key_str))
}

fn execute_set_paused(deps: DepsMut, info: MessageInfo, paused: bool) -> StdResult<Response> {
    ensure_owner(deps.as_ref(), &info)?;
    PAUSED.save(deps.storage, &paused)?;
    Ok(Response::new()
        .add_attribute("method", if paused { "pause" } else { "unpause" })
        .add_attribute("paused", paused.to_string()))
}

fn execute_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> StdResult<Response> {
    ensure_owner(deps.as_ref(), &info)?;
    let new_owner_addr = deps.api.addr_validate(&new_owner)?;
    PENDING_OWNER.save(deps.storage, &new_owner_addr)?;
    Ok(Response::new()
        .add_attribute("method", "transfer_ownership")
        .add_attribute("pending_owner", new_owner_addr))
}

fn execute_accept_ownership(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let pending = PENDING_OWNER
        .may_load(deps.storage)?
        .ok_or_else(|| StdError::msg("no pending owner"))?;
    if info.sender != pending {
        return Err(StdError::msg("Unauthorized: only pending owner can accept"));
    }
    OWNER.save(deps.storage, &pending)?;
    PENDING_OWNER.remove(deps.storage);
    Ok(Response::new()
        .add_attribute("method", "accept_ownership")
        .add_attribute("owner", pending))
}

fn execute_set_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> StdResult<Response> {
    ensure_admin(deps.as_ref(), &info)?;
    let new_admin_addr = deps.api.addr_validate(&new_admin)?;
    PENDING_ADMIN.save(deps.storage, &new_admin_addr)?;
    Ok(Response::new()
        .add_attribute("method", "set_admin")
        .add_attribute("pending_admin", new_admin_addr))
}

fn execute_accept_admin(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let pending = PENDING_ADMIN
        .may_load(deps.storage)?
        .ok_or_else(|| StdError::msg("no pending admin"))?;
    if info.sender != pending {
        return Err(StdError::msg("Unauthorized: only pending admin can accept"));
    }
    ADMIN.save(deps.storage, &pending)?;
    PENDING_ADMIN.remove(deps.storage);
    Ok(Response::new()
        .add_attribute("method", "accept_admin")
        .add_attribute("admin", pending))
}

fn execute_set_quorum_threshold(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    numerator: Uint256,
    denominator: Uint256,
) -> StdResult<Response> {
    ensure_admin(deps.as_ref(), &info)?;
    validate_quorum_params(numerator, denominator)?;
    QUORUM_NUMERATOR.save(deps.storage, &numerator, env.block.height)?;
    QUORUM_DENOMINATOR.save(deps.storage, &denominator, env.block.height)?;
    Ok(Response::new()
        .add_attribute("method", "wavs_set_quorum_threshold")
        .add_attribute("numerator", numerator.to_string())
        .add_attribute("denominator", denominator.to_string()))
}

fn execute_set_service_uri(
    deps: DepsMut,
    info: MessageInfo,
    service_uri: String,
) -> StdResult<Response> {
    ensure_admin(deps.as_ref(), &info)?;
    SERVICE_URI.save(deps.storage, &service_uri)?;
    Ok(Response::new().add_event(WavsServiceUriUpdatedEvent { service_uri }))
}

fn validate_quorum_params(numerator: Uint256, denominator: Uint256) -> StdResult<()> {
    if numerator.is_zero() || denominator.is_zero() || numerator > denominator {
        return Err(StdError::msg(
            "Invalid quorum parameters: numerator must be > 0, denominator must be > 0, and numerator must be <= denominator",
        ));
    }
    Ok(())
}

fn signing_key_is_zero(key: &EvmAddr) -> bool {
    key.as_bytes().iter().all(|b| *b == 0)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Owner {} => to_json_binary(&OWNER.load(deps.storage)?.to_string()),
        QueryMsg::PendingOwner {} => {
            to_json_binary(&PENDING_OWNER.may_load(deps.storage)?.map(|a| a.to_string()))
        }
        QueryMsg::Admin {} => to_json_binary(&ADMIN.load(deps.storage)?.to_string()),
        QueryMsg::PendingAdmin {} => {
            to_json_binary(&PENDING_ADMIN.may_load(deps.storage)?.map(|a| a.to_string()))
        }
        QueryMsg::Paused {} => to_json_binary(&PAUSED.load(deps.storage)?),
        QueryMsg::TotalWeight { reference_block } => {
            let weight = match reference_block {
                Some(h) => TOTAL_WEIGHT
                    .may_load_at_height(deps.storage, h)?
                    .unwrap_or_default(),
                None => TOTAL_WEIGHT.load(deps.storage)?,
            };
            to_json_binary(&weight)
        }
        QueryMsg::OperatorWeight {
            operator,
            reference_block,
        } => {
            let weight = match reference_block {
                Some(h) => OPERATOR_WEIGHTS
                    .may_load_at_height(deps.storage, operator, h)?
                    .unwrap_or_default(),
                None => OPERATOR_WEIGHTS
                    .may_load(deps.storage, operator)?
                    .unwrap_or_default(),
            };
            to_json_binary(&weight)
        }
        QueryMsg::OperatorSigningKey {
            operator,
            reference_block,
        } => {
            let key = match reference_block {
                Some(h) => OPERATOR_TO_SIGNING_KEY.may_load_at_height(deps.storage, operator, h)?,
                None => OPERATOR_TO_SIGNING_KEY.may_load(deps.storage, operator)?,
            };
            to_json_binary(&key)
        }
        QueryMsg::OperatorRegistered { operator } => to_json_binary(
            &OPERATOR_REGISTERED
                .may_load(deps.storage, operator)?
                .unwrap_or(false),
        ),

        QueryMsg::Wavs(msg) => match msg {
            ServiceManagerQueryMessages::WavsQuorumThreshold {} => {
                let numerator = QUORUM_NUMERATOR.load(deps.storage)?;
                let denominator = QUORUM_DENOMINATOR.load(deps.storage)?;
                let threshold =
                    wavs_types::contracts::cosmwasm::service_manager::QuorumThreshold {
                        numerator,
                        denominator,
                    };
                to_json_binary(&threshold)
            }
            ServiceManagerQueryMessages::WavsOperatorWeight { operator_address } => {
                // For ECDSA standalone, operator_address is interpreted as the signing key.
                let signing_key_str = operator_address.to_string();
                let operator =
                    SIGNING_KEY_TO_OPERATOR.may_load(deps.storage, signing_key_str)?;
                let weight = match operator {
                    Some(op) => OPERATOR_WEIGHTS
                        .may_load(deps.storage, op.to_string())?
                        .unwrap_or_default(),
                    None => Uint256::zero(),
                };
                to_json_binary(&weight)
            }
            ServiceManagerQueryMessages::WavsValidate {
                envelope,
                signature_data,
            } => {
                let result = wavs_validate(deps, envelope, signature_data)?;
                to_json_binary(&result)
            }
            ServiceManagerQueryMessages::WavsServiceUri {} => {
                let uri = SERVICE_URI
                    .may_load(deps.storage)?
                    .unwrap_or_default();
                to_json_binary(&uri)
            }
            ServiceManagerQueryMessages::WavsLatestOperatorForSigningKey { signing_key_addr } => {
                let signing_key_str = signing_key_addr.to_string();
                let operator =
                    SIGNING_KEY_TO_OPERATOR.may_load(deps.storage, signing_key_str)?;
                // Convert Addr -> EvmAddr-shape isn't applicable here (operators are
                // CW addresses); return None for non-EVM operators. This query is
                // primarily meaningful for the mirror family.
                let _ = operator;
                to_json_binary::<Option<EvmAddr>>(&None)
            }
        },
    }
}

/// Top-level WavsValidate entry: verifies signatures, accumulates weights at
/// the reference block, and runs the quorum check.
fn wavs_validate(
    deps: Deps,
    envelope: WavsEnvelope,
    signature_data: WavsSignatureData,
) -> StdResult<WavsValidateResult> {
    if PAUSED.load(deps.storage)? {
        return Ok(WavsValidateResult::Err(WavsValidateError::InvalidSignature(
            "contract is paused".to_string(),
        )));
    }

    if signature_data.signers.is_empty()
        || signature_data.signers.len() != signature_data.signatures.len()
    {
        return Ok(WavsValidateResult::Err(
            WavsValidateError::InvalidSignatureLength,
        ));
    }

    let reference_block = signature_data.reference_block as u64;

    // Snapshot reads: total weight + quorum threshold both pinned to
    // reference_block (audit M-4 preventative for new code).
    let total_weight = TOTAL_WEIGHT
        .may_load_at_height(deps.storage, reference_block)?
        .unwrap_or_default();
    let numerator = QUORUM_NUMERATOR
        .may_load_at_height(deps.storage, reference_block)?
        .ok_or_else(|| StdError::msg("quorum numerator missing at reference block"))?;
    let denominator = QUORUM_DENOMINATOR
        .may_load_at_height(deps.storage, reference_block)?
        .ok_or_else(|| StdError::msg("quorum denominator missing at reference block"))?;

    if total_weight.is_zero() {
        return Ok(WavsValidateResult::Err(
            WavsValidateError::InsufficientQuorumZero,
        ));
    }

    let mut signed_weight = Uint256::zero();
    let mut seen: HashSet<[u8; 20]> = HashSet::new();

    for (i, signing_key) in signature_data.signers.iter().enumerate() {
        if signing_key_is_zero(signing_key) {
            return Ok(WavsValidateResult::Err(WavsValidateError::InvalidSignature(
                "signing key is zero".to_string(),
            )));
        }
        if !seen.insert(signing_key.as_bytes()) {
            return Ok(WavsValidateResult::Err(WavsValidateError::InvalidSignature(
                format!("duplicate signing key: {signing_key}"),
            )));
        }

        let signing_key_str = signing_key.to_string();

        // No fallback to latest — H-2 fix preventatively applied to new code.
        // If a signing key wasn't registered as of reference_block, the
        // signer doesn't count.
        let operator = match SIGNING_KEY_TO_OPERATOR.may_load_at_height(
            deps.storage,
            signing_key_str.clone(),
            reference_block,
        )? {
            Some(op) => op,
            None => {
                return Ok(WavsValidateResult::Err(WavsValidateError::InvalidSignature(
                    format!("signer not registered at reference_block: {signing_key}"),
                )));
            }
        };

        let signature = &signature_data.signatures[i];
        match is_valid_signature(deps, &envelope, signature, signing_key) {
            Ok(true) => {}
            Ok(false) => {
                return Ok(WavsValidateResult::Err(WavsValidateError::InvalidSignature(
                    format!("invalid signature from {signing_key}"),
                )));
            }
            Err(e) => {
                return Ok(WavsValidateResult::Err(WavsValidateError::InvalidSignature(
                    e.to_string(),
                )));
            }
        }

        let operator_weight = OPERATOR_WEIGHTS
            .may_load_at_height(deps.storage, operator.to_string(), reference_block)?
            .unwrap_or_default();
        if operator_weight.is_zero() {
            return Ok(WavsValidateResult::Err(WavsValidateError::InvalidSignature(
                format!("operator {operator} has zero weight at reference_block"),
            )));
        }
        signed_weight = signed_weight
            .checked_add(operator_weight)
            .map_err(|e| StdError::msg(format!("signed weight overflow: {e}")))?;
    }

    // Quorum check via Uint512: signed * denominator >= total * numerator.
    let lhs = Uint512::from(signed_weight) * Uint512::from(denominator);
    let rhs = Uint512::from(total_weight) * Uint512::from(numerator);
    if lhs < rhs {
        let threshold = total_weight.full_mul(numerator) / Uint512::from(denominator);
        let threshold_u256: Uint256 = threshold.try_into().unwrap_or(Uint256::MAX);
        return Ok(WavsValidateResult::Err(
            WavsValidateError::InsufficientQuorum {
                signer_weight: signed_weight,
                threshold_weight: threshold_u256,
                total_weight,
            },
        ));
    }

    Ok(WavsValidateResult::Ok)
}

/// secp256k1 recovery + EIP-191 digest contract — mirrors POA's
/// `signer.isValidSignatureNow(keccak256(envelope), sig)` behavior and is the
/// canonical port of `mirror::stake_registry::is_valid_signature`. Cross-chain
/// vector test pins this format (Phase 7a).
fn is_valid_signature(
    deps: Deps,
    envelope: &WavsEnvelope,
    signature: &HexBinary,
    signer_address: &EvmAddr,
) -> StdResult<bool> {
    if signature.len() != SECP256K1_SIGNATURE_LEN {
        return Ok(false);
    }
    if signature.as_slice().iter().all(|&b| b == 0) {
        return Ok(false);
    }

    let sig_bytes = signature.as_slice();
    let rs = &sig_bytes[0..64];
    let recovery_id = sig_bytes[64];
    // Normalize Ethereum-format recovery id (27/28) to k256 format (0/1).
    let normalized_recovery_id = match recovery_id {
        27 => 0,
        28 => 1,
        0 | 1 => recovery_id,
        _ => return Ok(false),
    };

    let hash = eip191_hash_message(keccak256(envelope.as_slice()));

    let pubkey = deps
        .api
        .secp256k1_recover_pubkey(hash.as_slice(), rs, normalized_recovery_id)?;

    if !deps.api.secp256k1_verify(hash.as_slice(), rs, &pubkey)? {
        return Ok(false);
    }

    let recovered = ethereum_address_raw(&pubkey)?;
    Ok(&recovered[..] == signer_address.as_bytes().as_slice())
}

fn ethereum_address_raw(pubkey: &[u8]) -> StdResult<[u8; 20]> {
    let (tag, data) = pubkey
        .split_first()
        .ok_or_else(|| StdError::msg("Public key must not be empty"))?;
    if *tag != 0x04 {
        return Err(StdError::msg("Public key must start with 0x04"));
    }
    if data.len() != 64 {
        return Err(StdError::msg("Public key must be 65 bytes long"));
    }
    let hash = Keccak256::digest(data);
    let out: [u8; 20] = hash[hash.len() - 20..]
        .try_into()
        .map_err(|_| StdError::msg("keccak digest tail not 20 bytes"))?;
    Ok(out)
}

// `Event` import retained for forward use by registration events. Suppress
// an unused-import warning until the next commit lights up event emission.
#[allow(dead_code)]
fn _unused_event_marker(_: Event) {}
