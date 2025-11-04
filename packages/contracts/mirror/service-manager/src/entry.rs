use cosmwasm_std::{
    entry_point, to_json_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdResult, Uint256,
};
use cw2::set_contract_version;
use layer_climb_address::EvmAddr;
use wavs_types::contracts::cosmwasm::service_manager::{
    error::WavsValidateError, event::WavsServiceUriUpdatedEvent, ServiceManagerExecuteMessages,
    ServiceManagerQueryMessages, WavsValidateResult,
};

use crate::state::{self, ADMIN, STAKE_REGISTRY};
use cw_wavs_mirror_api::service_manager::{ExecuteMsg, InstantiateMsg, QueryMsg};

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

    let admin = deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(deps.storage, &admin)?;
    STAKE_REGISTRY.save(deps.storage, &info.sender)?;

    Ok(Response::default().add_attribute("admin", admin))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Wavs(msg) => match msg {
            ServiceManagerExecuteMessages::WavsSetServiceUri { service_uri } => {
                let admin = ADMIN.load(deps.storage)?;
                if _info.sender != admin {
                    return Err(cosmwasm_std::StdError::msg(
                        "Unauthorized: only admin can set service URI",
                    ));
                }

                state::SERVICE_URI.save(deps.storage, &service_uri)?;

                Ok(Response::new().add_event(WavsServiceUriUpdatedEvent { service_uri }))
            }
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Mirror(msg) => match msg {
            cw_wavs_mirror_api::service_manager::MirrorServiceManagerQueryMessages::Admin {} => {
                let admin = ADMIN.load(deps.storage)?;
                to_json_binary(&admin)
            }
        },
        QueryMsg::Wavs(msg) => match msg {
            ServiceManagerQueryMessages::WavsOperatorWeight { operator_address } => {
                // Query stake registry for operator weight
                let stake_registry = state::STAKE_REGISTRY.load(deps.storage)?;
                let weight: Uint256 = deps.querier.query_wasm_smart(
                    stake_registry,
                    &cw_wavs_mirror_api::stake_registry::QueryMsg::GetOperatorWeight {
                        operator: operator_address,
                    },
                )?;
                to_json_binary(&weight)
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
                            WavsValidateError::MissingRegistry,
                        ))
                    }
                };

                // Query stake registry
                let res: cw_wavs_mirror_api::stake_registry::ValidationResult =
                    deps.querier.query_wasm_smart(
                        stake_registry,
                        &cw_wavs_mirror_api::stake_registry::QueryMsg::ValidateSignature {
                            envelope,
                            signature_data,
                        },
                    )?;

                match res.error {
                    Some(error) => to_json_binary(&WavsValidateResult::Err(error)),
                    None => to_json_binary(&WavsValidateResult::Ok),
                }
            }
            ServiceManagerQueryMessages::WavsServiceUri {} => {
                to_json_binary(&state::SERVICE_URI.load(deps.storage)?)
            }
            ServiceManagerQueryMessages::WavsLatestOperatorForSigningKey { signing_key_addr } => {
                // Query stake registry for operator by signing key
                let stake_registry = state::STAKE_REGISTRY.load(deps.storage)?;
                let operator: Option<EvmAddr> = deps.querier.query_wasm_smart(
                    stake_registry,
                    &cw_wavs_mirror_api::stake_registry::QueryMsg::GetLatestOperatorForSigningKey {
                        signing_key: signing_key_addr,
                    },
                )?;
                to_json_binary(&operator)
            }
        },
    }
}
