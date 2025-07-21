use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
};
use cw_multi_test::{Contract, ContractWrapper};
use cw_wavs_types::{ServiceManagerExecuteMsg, ServiceManagerQueryMsg};

fn mock_service_manager_instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    Ok(Response::new())
}

fn mock_service_manager_execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ServiceManagerExecuteMsg,
) -> StdResult<Response> {
    Ok(Response::new())
}

fn mock_service_manager_query_validate_success(
    _deps: Deps,
    _env: Env,
    msg: ServiceManagerQueryMsg,
) -> StdResult<Binary> {
    match msg {
        ServiceManagerQueryMsg::Validate { .. } => Ok(to_json_binary(&())?),
        _ => Err(StdError::msg("invalid query")),
    }
}

fn mock_service_manager_query_validation_error(
    _deps: Deps,
    _env: Env,
    msg: ServiceManagerQueryMsg,
) -> StdResult<Binary> {
    match msg {
        ServiceManagerQueryMsg::Validate { .. } => Err(StdError::msg("validation error")),
        _ => Err(StdError::msg("invalid query")),
    }
}

pub fn mock_service_manager_contract_validate_success() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mock_service_manager_execute,
        mock_service_manager_instantiate,
        mock_service_manager_query_validate_success,
    );
    Box::new(contract)
}

pub fn mock_service_manager_contract_validation_error() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mock_service_manager_execute,
        mock_service_manager_instantiate,
        mock_service_manager_query_validation_error,
    );
    Box::new(contract)
}
