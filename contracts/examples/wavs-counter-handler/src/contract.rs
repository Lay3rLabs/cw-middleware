#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use cw_wavs_types::{validate_signed_envelope, Envelope, SignatureData};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{COUNTER, SERVICE_MANAGER};

const CONTRACT_NAME: &str = "crates.io:wavs-counter-handler";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Store the service manager address
    let service_manager = deps.api.addr_validate(&msg.service_manager)?;
    SERVICE_MANAGER.save(deps.storage, &service_manager)?;

    COUNTER.save(deps.storage, &0)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::HandleSignedEnvelope {
            envelope,
            signature_data,
        } => handle_signed_envelope(deps, envelope, signature_data),
    }
}

fn handle_signed_envelope(
    deps: DepsMut,
    envelope: Envelope,
    signature_data: SignatureData,
) -> Result<Response, ContractError> {
    // Validate the signature data
    validate_signed_envelope(
        &deps.as_ref(),
        &SERVICE_MANAGER.load(deps.storage)?,
        &envelope,
        signature_data,
    )?;

    let counter = COUNTER.update(deps.storage, |counter| Ok::<u32, StdError>(counter + 1))?;

    Ok(Response::new()
        .add_attribute("action", "handle_signed_envelope")
        .add_attribute("counter", counter.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ServiceManager {} => {
            let service_manager = SERVICE_MANAGER.load(deps.storage)?;
            to_json_binary(&service_manager)
        }
        QueryMsg::Info {} => {
            let contract_version = get_contract_version(deps.storage)?;
            to_json_binary(&contract_version)
        }
        QueryMsg::Counter {} => {
            let counter = COUNTER.load(deps.storage)?;
            to_json_binary(&counter)
        }
    }
}
