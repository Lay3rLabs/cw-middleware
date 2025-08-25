use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdResult,
};
use cw2::set_contract_version;
use wavs_types::contracts::cosmwasm::service_manager::{
    error::WavsValidateError, event::WavsServiceUriUpdatedEvent, ServiceManagerExecuteMessages,
    ServiceManagerQueryMessages, WavsValidateResult,
};

use crate::state::{self, STAKE_REGISTRY};
use alloy_primitives::keccak256 as alloy_keccak256;
use ethabi::{encode, Token};
use mirror_api::service_manager::{
    ExecuteMsg, InstantiateMsg, OwnableExecuteMsg, OwnableQueryMsg, QueryMsg,
};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let ownership = cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;
    STAKE_REGISTRY.save(deps.storage, &info.sender)?;

    Ok(Response::default().add_attributes(ownership.into_attributes()))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Wavs(msg) => match msg {
            ServiceManagerExecuteMessages::WavsSetServiceUri { service_uri } => {
                state::SERVICE_URI.save(deps.storage, &service_uri)?;

                Ok(Response::new().add_event(WavsServiceUriUpdatedEvent { service_uri }))
            }
        },
        ExecuteMsg::SetSigningKey {
            operator,
            signing_key,
            weight,
        } => {
            cw_ownable::assert_owner(deps.storage, &info.sender)?;
            state::OPERATOR_SIGNING_KEY_ADDRS.save(deps.storage, &operator, &signing_key)?;
            state::OPERATOR_WEIGHTS.save(deps.storage, &operator, &weight)?;
            Ok(Response::default())
        }
        ExecuteMsg::Ownable(msg) => match msg {
            OwnableExecuteMsg::UpdateOwnership(action) => {
                let ownership =
                    cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;

                Ok(Response::default().add_attributes(ownership.into_attributes()))
            }
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Wavs(msg) => match msg {
            ServiceManagerQueryMessages::WavsOperatorWeight { operator_address } => {
                // TODO: integrate with mirror stake registry for operator weight queries
                to_json_binary(&state::OPERATOR_WEIGHTS.load(deps.storage, &operator_address)?)
            }
            ServiceManagerQueryMessages::WavsValidate {
                envelope,
                signature_data,
            } => {
                // Validate signatures via stake registry, if configured
                let stake_registry = match state::STAKE_REGISTRY.may_load(deps.storage)? {
                    Some(addr) => addr,
                    None => {
                        return to_json_binary(&WavsValidateResult::Err(
                            WavsValidateError::InvalidSignature,
                        ))
                    }
                };

                // Compute digest from envelope payload (keccak256 of payload bytes)
                let decoded = match envelope.decode() {
                    Ok(d) => d,
                    Err(_) => {
                        return to_json_binary(&WavsValidateResult::Err(
                            WavsValidateError::InvalidSignature,
                        ))
                    }
                };
                let digest_b256 = alloy_keccak256(&decoded.payload);
                let digest_bin = Binary::from(digest_b256.to_vec());

                // Build ABI-encoded signature data (address[] signers, bytes[] signatures, uint32 referenceBlock)
                // Use the provided reference_block from signature_data.
                let signers_tokens: Vec<Token> = signature_data
                    .signers
                    .iter()
                    .map(|s| Token::Address(s.as_bytes().into()))
                    .collect();
                let sigs_tokens: Vec<Token> = signature_data
                    .signatures
                    .iter()
                    .map(|sig| Token::Bytes(sig.to_vec()))
                    .collect();
                let encoded = encode(&[
                    Token::Array(signers_tokens),
                    Token::Array(sigs_tokens),
                    Token::Uint(signature_data.reference_block.into()),
                ]);
                let encoded_bin = Binary::from(encoded);

                // Query stake registry
                let res: mirror_api::stake_registry::ValidationResult =
                    deps.querier.query_wasm_smart(
                        stake_registry,
                        &mirror_api::stake_registry::QueryMsg::ValidateSignature {
                            digest: digest_bin,
                            signature_data: encoded_bin,
                        },
                    )?;

                if res.is_valid {
                    to_json_binary(&WavsValidateResult::Ok)
                } else {
                    to_json_binary(&WavsValidateResult::Err(
                        WavsValidateError::InvalidSignature,
                    ))
                }
            }
            ServiceManagerQueryMessages::WavsServiceUri {} => {
                to_json_binary(&state::SERVICE_URI.load(deps.storage)?)
            }
            ServiceManagerQueryMessages::WavsLatestOperatorForSigningKey { signing_key_addr } => {
                to_json_binary(
                    &state::OPERATOR_SIGNING_KEY_ADDRS.may_load(deps.storage, &signing_key_addr)?,
                )
            }
        },
        QueryMsg::Ownable(msg) => match msg {
            OwnableQueryMsg::Ownership {} => {
                cw_ownable::get_ownership(deps.storage).and_then(|x| to_json_binary(&x))
            }
        },
    }
}
