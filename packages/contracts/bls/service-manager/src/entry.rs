use std::collections::HashSet;

use alloy_primitives::Address as EthAddress;
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Deps, DepsMut, Env, HashFunction, HexBinary, MessageInfo,
    QueryResponse, Response, StdError, StdResult, Uint256, Uint512, BLS12_381_G1_GENERATOR,
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
    ADMIN, BLS_KEY_ID_TO_OPERATOR, OPERATOR_REGISTERED, OPERATOR_TO_BLS_KEY_ID,
    OPERATOR_TO_BLS_PUBKEY, OPERATOR_WEIGHTS, OWNER, PAUSED, PENDING_ADMIN, PENDING_OWNER,
    QUORUM_DENOMINATOR, QUORUM_NUMERATOR, SERVICE_URI, TOTAL_WEIGHT,
};
use cw_wavs_bls_api::service_manager::{ExecuteMsg, InstantiateMsg, QueryMsg};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// BLS12-381 G1 compressed pubkey length (per cosmwasm-crypto convention).
const G1_COMPRESSED_LEN: usize = 48;
/// BLS12-381 G2 compressed signature length.
const G2_COMPRESSED_LEN: usize = 96;
/// RFC 9380 standard DST for BLS12-381 minimal-pubkey-size signatures over
/// G2 — byte-for-byte match with poa-middleware/contracts/src/bls/libs/HashToCurve.sol.
const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_";

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
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::RegisterOperator {
            operator,
            bls_pubkey,
            weight,
        } => execute_register_operator(deps, env, info, operator, bls_pubkey, weight),
        ExecuteMsg::DeregisterOperator { operator } => {
            execute_deregister_operator(deps, env, info, operator)
        }
        ExecuteMsg::UpdateOperatorWeight { operator, weight } => {
            execute_update_operator_weight(deps, env, info, operator, weight)
        }
        ExecuteMsg::UpdateOperatorBlsPubkey {
            operator,
            new_bls_pubkey,
        } => execute_update_operator_bls_pubkey(deps, env, info, operator, new_bls_pubkey),

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

fn validate_quorum_params(numerator: Uint256, denominator: Uint256) -> StdResult<()> {
    if numerator.is_zero() || denominator.is_zero() || numerator > denominator {
        return Err(StdError::msg(
            "Invalid quorum parameters: numerator must be > 0, denominator must be > 0, and numerator must be <= denominator",
        ));
    }
    Ok(())
}

/// Derive the 20-byte BLS-key id from a 48-byte G1 compressed pubkey.
/// `keccak256(pubkey)[..20]`.
fn bls_key_id(pubkey: &[u8]) -> StdResult<EvmAddr> {
    if pubkey.len() != G1_COMPRESSED_LEN {
        return Err(StdError::msg(format!(
            "BLS pubkey must be {G1_COMPRESSED_LEN} bytes (compressed G1)"
        )));
    }
    let hash = Keccak256::digest(pubkey);
    let id_bytes: [u8; 20] = hash[hash.len() - 20..]
        .try_into()
        .map_err(|_| StdError::msg("keccak digest tail not 20 bytes"))?;
    Ok(EvmAddr::from(EthAddress::from(id_bytes)))
}

fn pubkey_is_zero(pubkey: &[u8]) -> bool {
    pubkey.iter().all(|b| *b == 0)
}

fn execute_register_operator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    bls_pubkey: HexBinary,
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

    if pubkey_is_zero(bls_pubkey.as_slice()) {
        return Err(StdError::msg("BLS pubkey is zero"));
    }
    let key_id = bls_key_id(bls_pubkey.as_slice())?;
    let key_id_str = key_id.to_string();

    if BLS_KEY_ID_TO_OPERATOR
        .may_load(deps.storage, key_id_str.clone())?
        .is_some()
    {
        return Err(StdError::msg("BLS key id already in use"));
    }

    let snapshot_height = env.block.height;
    OPERATOR_REGISTERED.save(deps.storage, operator_key.clone(), &true)?;
    OPERATOR_WEIGHTS.save(deps.storage, operator_key.clone(), &weight, snapshot_height)?;
    OPERATOR_TO_BLS_PUBKEY.save(
        deps.storage,
        operator_key.clone(),
        &bls_pubkey,
        snapshot_height,
    )?;
    OPERATOR_TO_BLS_KEY_ID.save(deps.storage, operator_key.clone(), &key_id, snapshot_height)?;
    BLS_KEY_ID_TO_OPERATOR.save(
        deps.storage,
        key_id_str.clone(),
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
        .add_attribute("bls_key_id", key_id_str)
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
    let current_key_id = OPERATOR_TO_BLS_KEY_ID
        .may_load(deps.storage, operator_key.clone())?
        .ok_or_else(|| StdError::msg("operator has no BLS key id"))?;

    OPERATOR_WEIGHTS.remove(deps.storage, operator_key.clone(), snapshot_height)?;
    OPERATOR_TO_BLS_PUBKEY.remove(deps.storage, operator_key.clone(), snapshot_height)?;
    OPERATOR_TO_BLS_KEY_ID.remove(deps.storage, operator_key.clone(), snapshot_height)?;
    BLS_KEY_ID_TO_OPERATOR.remove(deps.storage, current_key_id.to_string(), snapshot_height)?;
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

fn execute_update_operator_bls_pubkey(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    new_bls_pubkey: HexBinary,
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
    if pubkey_is_zero(new_bls_pubkey.as_slice()) {
        return Err(StdError::msg("BLS pubkey is zero"));
    }
    let new_key_id = bls_key_id(new_bls_pubkey.as_slice())?;
    let new_key_id_str = new_key_id.to_string();

    if let Some(existing) = BLS_KEY_ID_TO_OPERATOR.may_load(deps.storage, new_key_id_str.clone())? {
        if existing != operator_addr {
            return Err(StdError::msg("BLS key id already in use"));
        }
    }

    let snapshot_height = env.block.height;
    let old_pubkey = OPERATOR_TO_BLS_PUBKEY
        .may_load(deps.storage, operator_key.clone())?
        .ok_or_else(|| StdError::msg("operator has no BLS pubkey"))?;
    let old_key_id = OPERATOR_TO_BLS_KEY_ID
        .may_load(deps.storage, operator_key.clone())?
        .ok_or_else(|| StdError::msg("operator has no BLS key id"))?;

    if old_pubkey == new_bls_pubkey {
        return Err(StdError::msg("BLS pubkey unchanged"));
    }

    BLS_KEY_ID_TO_OPERATOR.remove(deps.storage, old_key_id.to_string(), snapshot_height)?;
    OPERATOR_TO_BLS_PUBKEY.save(
        deps.storage,
        operator_key.clone(),
        &new_bls_pubkey,
        snapshot_height,
    )?;
    OPERATOR_TO_BLS_KEY_ID.save(
        deps.storage,
        operator_key.clone(),
        &new_key_id,
        snapshot_height,
    )?;
    BLS_KEY_ID_TO_OPERATOR.save(
        deps.storage,
        new_key_id_str.clone(),
        &operator_addr,
        snapshot_height,
    )?;

    Ok(Response::new()
        .add_attribute("method", "update_operator_bls_pubkey")
        .add_attribute("operator", operator_key)
        .add_attribute("old_bls_key_id", old_key_id.to_string())
        .add_attribute("new_bls_key_id", new_key_id_str))
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

fn execute_set_admin(deps: DepsMut, info: MessageInfo, new_admin: String) -> StdResult<Response> {
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
        QueryMsg::OperatorBlsPubkey {
            operator,
            reference_block,
        } => {
            let key = match reference_block {
                Some(h) => OPERATOR_TO_BLS_PUBKEY.may_load_at_height(deps.storage, operator, h)?,
                None => OPERATOR_TO_BLS_PUBKEY.may_load(deps.storage, operator)?,
            };
            to_json_binary(&key)
        }
        QueryMsg::OperatorSigningKeyId {
            operator,
            reference_block,
        } => {
            let id = match reference_block {
                Some(h) => OPERATOR_TO_BLS_KEY_ID.may_load_at_height(deps.storage, operator, h)?,
                None => OPERATOR_TO_BLS_KEY_ID.may_load(deps.storage, operator)?,
            };
            to_json_binary(&id)
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
                let threshold = wavs_types::contracts::cosmwasm::service_manager::QuorumThreshold {
                    numerator,
                    denominator,
                };
                to_json_binary(&threshold)
            }
            ServiceManagerQueryMessages::WavsOperatorWeight { operator_address } => {
                let id_str = operator_address.to_string();
                let operator = BLS_KEY_ID_TO_OPERATOR.may_load(deps.storage, id_str)?;
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
                let uri = SERVICE_URI.may_load(deps.storage)?.unwrap_or_default();
                to_json_binary(&uri)
            }
            ServiceManagerQueryMessages::WavsLatestOperatorForSigningKey { signing_key_addr } => {
                // For the BLS family, the "operator address" returned would
                // be a CW bech32 (Addr), not an EvmAddr. The wavs-types
                // schema returns Option<EvmAddr>; return None for BLS since
                // the natural response type doesn't fit.
                let _ = signing_key_addr;
                to_json_binary::<Option<EvmAddr>>(&None)
            }
        },
    }
}

/// Validate a BLS-aggregate-signed envelope. The wavs-types signature
/// format is awkward for BLS: `signers` carries 20-byte BLS-key ids
/// (= keccak256(g1_pubkey)[..20]) instead of full pubkeys, and
/// `signatures` carries exactly one entry — the 96-byte compressed G2
/// aggregate signature.
fn wavs_validate(
    deps: Deps,
    envelope: WavsEnvelope,
    signature_data: WavsSignatureData,
) -> StdResult<WavsValidateResult> {
    if PAUSED.load(deps.storage)? {
        return Ok(WavsValidateResult::Err(
            WavsValidateError::InvalidSignature("contract is paused".to_string()),
        ));
    }

    if signature_data.signers.is_empty() {
        return Ok(WavsValidateResult::Err(
            WavsValidateError::InvalidSignatureLength,
        ));
    }
    if signature_data.signatures.len() != 1 {
        return Ok(WavsValidateResult::Err(
            WavsValidateError::InvalidSignature(format!(
                "BLS expects 1 aggregate signature, got {}",
                signature_data.signatures.len()
            )),
        ));
    }
    let aggregate_sig = &signature_data.signatures[0];
    if aggregate_sig.len() != G2_COMPRESSED_LEN {
        return Ok(WavsValidateResult::Err(
            WavsValidateError::InvalidSignature(format!(
                "aggregate signature must be {G2_COMPRESSED_LEN} bytes (compressed G2), got {}",
                aggregate_sig.len()
            )),
        ));
    }

    let reference_block = signature_data.reference_block as u64;
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

    // Concatenate signer pubkeys for batched G1 aggregation. Also enforce
    // sorted-ascending uniqueness on the 20-byte ids.
    let mut signed_weight = Uint256::zero();
    let mut seen: HashSet<[u8; 20]> = HashSet::new();
    let mut last_id: Option<[u8; 20]> = None;
    let mut concatenated_pubkeys: Vec<u8> =
        Vec::with_capacity(signature_data.signers.len() * G1_COMPRESSED_LEN);

    for signer_id in &signature_data.signers {
        let id_bytes: [u8; 20] = signer_id.as_bytes();
        if id_bytes.iter().all(|b| *b == 0) {
            return Ok(WavsValidateResult::Err(
                WavsValidateError::InvalidSignature("BLS-key id is zero".to_string()),
            ));
        }
        if !seen.insert(id_bytes) {
            return Ok(WavsValidateResult::Err(
                WavsValidateError::InvalidSignature(format!("duplicate BLS-key id: {signer_id}")),
            ));
        }
        if let Some(prev) = last_id {
            if id_bytes <= prev {
                return Ok(WavsValidateResult::Err(
                    WavsValidateError::InvalidSignature(
                        "BLS-key ids must be strictly sorted ascending".to_string(),
                    ),
                ));
            }
        }
        last_id = Some(id_bytes);

        let id_str = signer_id.to_string();
        let operator = match BLS_KEY_ID_TO_OPERATOR.may_load_at_height(
            deps.storage,
            id_str.clone(),
            reference_block,
        )? {
            Some(op) => op,
            None => {
                return Ok(WavsValidateResult::Err(
                    WavsValidateError::InvalidSignature(format!(
                        "BLS-key id {signer_id} not registered at reference_block"
                    )),
                ));
            }
        };

        let pubkey = match OPERATOR_TO_BLS_PUBKEY.may_load_at_height(
            deps.storage,
            operator.to_string(),
            reference_block,
        )? {
            Some(pk) => pk,
            None => {
                return Ok(WavsValidateResult::Err(
                    WavsValidateError::InvalidSignature(format!(
                        "operator {operator} has no pubkey at reference_block"
                    )),
                ));
            }
        };
        if pubkey.len() != G1_COMPRESSED_LEN {
            return Ok(WavsValidateResult::Err(
                WavsValidateError::InvalidSignature(format!(
                    "operator {operator} pubkey wrong length"
                )),
            ));
        }
        concatenated_pubkeys.extend_from_slice(pubkey.as_slice());

        let operator_weight = OPERATOR_WEIGHTS
            .may_load_at_height(deps.storage, operator.to_string(), reference_block)?
            .unwrap_or_default();
        if operator_weight.is_zero() {
            return Ok(WavsValidateResult::Err(
                WavsValidateError::InvalidSignature(format!(
                    "operator {operator} has zero weight at reference_block"
                )),
            ));
        }
        signed_weight = signed_weight
            .checked_add(operator_weight)
            .map_err(|e| StdError::msg(format!("signed weight overflow: {e}")))?;
    }

    // Aggregate G1 pubkeys via the host call.
    let aggregated_pubkey = match deps.api.bls12_381_aggregate_g1(&concatenated_pubkeys) {
        Ok(pk) => pk,
        Err(e) => {
            return Ok(WavsValidateResult::Err(
                WavsValidateError::InvalidSignature(format!("g1 aggregation failed: {e}")),
            ));
        }
    };

    // Hash the envelope to G2 with the locked-in DST. Off-chain signers
    // must use the same DST byte-for-byte: see audit Decision #7.
    let message_g2 =
        match deps
            .api
            .bls12_381_hash_to_g2(HashFunction::Sha256, envelope.as_slice(), DST)
        {
            Ok(p) => p,
            Err(e) => {
                return Ok(WavsValidateResult::Err(
                    WavsValidateError::InvalidSignature(format!("hash_to_g2 failed: {e}")),
                ));
            }
        };

    // Pairing equality: e(G1::generator, sig) == e(aggregated_pubkey, H(m)).
    // Per cosmwasm-std doc: bls12_381_pairing_equality(ps, qs, r, s)
    // checks e(ps, qs) == e(r, s).
    let valid = match deps.api.bls12_381_pairing_equality(
        &BLS12_381_G1_GENERATOR,
        aggregate_sig.as_slice(),
        &aggregated_pubkey,
        &message_g2,
    ) {
        Ok(v) => v,
        Err(e) => {
            return Ok(WavsValidateResult::Err(
                WavsValidateError::InvalidSignature(format!("pairing equality failed: {e}")),
            ));
        }
    };
    if !valid {
        return Ok(WavsValidateResult::Err(
            WavsValidateError::InvalidSignature(
                "aggregate signature failed pairing check".to_string(),
            ),
        ));
    }

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
