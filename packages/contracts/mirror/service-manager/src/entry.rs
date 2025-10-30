use cosmwasm_std::{
    entry_point, to_json_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdResult,
};
use cw2::set_contract_version;
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

    let admin = deps.api.addr_validate(&msg.owner)?;
    ADMIN.save(deps.storage, &admin)?;
    STAKE_REGISTRY.save(deps.storage, &info.sender)?;

    Ok(Response::default().add_attribute("admin", admin))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
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
            let admin = ADMIN.load(deps.storage)?;
            if info.sender != admin {
                return Err(cosmwasm_std::StdError::msg("Unauthorized"));
            }
            state::SIGNING_KEY_TO_OPERATOR.save(deps.storage, &signing_key, &operator)?;
            state::OPERATOR_WEIGHTS.save(deps.storage, &operator, &weight)?;
            Ok(Response::default())
        }
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

                // Query stake registry
                let res: cw_wavs_mirror_api::stake_registry::ValidationResult =
                    deps.querier.query_wasm_smart(
                        stake_registry,
                        &cw_wavs_mirror_api::stake_registry::QueryMsg::ValidateSignature {
                            envelope,
                            signature_data,
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
                    &state::SIGNING_KEY_TO_OPERATOR.may_load(deps.storage, &signing_key_addr)?,
                )
            }
        },
    }
}
