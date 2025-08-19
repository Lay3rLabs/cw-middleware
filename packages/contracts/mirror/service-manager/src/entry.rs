use cosmwasm_std::{
    entry_point, to_json_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdResult,
};
use cw2::set_contract_version;
use wavs_types::contracts::cosmwasm::service_manager::{
    error::WavsValidateError, event::WavsServiceUriUpdatedEvent, ServiceManagerExecuteMessages,
    ServiceManagerQueryMessages, WavsValidateResult,
};

use crate::state;
use mirror_api::service_manager::{ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Initialize admin
    let admin = deps
        .api
        .addr_validate(&msg.admin)
        .map_err(|_| cosmwasm_std::StdError::msg("Invalid admin address"))?;
    state::ADMIN.save(deps.storage, &admin)?;

    Ok(Response::default())
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
            // Only admin can set signing keys
            let admin = state::ADMIN.load(deps.storage)?;
            if info.sender != admin {
                return Err(cosmwasm_std::StdError::msg("Unauthorized"));
            }
            state::OPERATOR_SIGNING_KEY_ADDRS.save(deps.storage, &operator, &signing_key)?;
            state::OPERATOR_WEIGHTS.save(deps.storage, &operator, &weight)?;
            Ok(Response::default())
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Wavs(msg) => match msg {
            ServiceManagerQueryMessages::WavsOperatorWeight { operator_address } => {
                // TODO: query stake registry etc.
                to_json_binary(&state::OPERATOR_WEIGHTS.load(deps.storage, &operator_address)?)
            }
            ServiceManagerQueryMessages::WavsValidate {
                envelope: _,
                signature_data,
            } => {
                // TODO: real validation logic
                for signer in &signature_data.signers {
                    let _operator_addr =
                        match state::OPERATOR_SIGNING_KEY_ADDRS.load(deps.storage, signer) {
                            Ok(addr) => addr,
                            Err(_) => {
                                return to_json_binary(&WavsValidateResult::Err(
                                    WavsValidateError::InvalidSignature,
                                ));
                            }
                        };
                }
                to_json_binary(&WavsValidateResult::Ok)
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
    }
}
