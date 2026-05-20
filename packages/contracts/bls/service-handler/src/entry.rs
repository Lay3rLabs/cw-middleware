use cosmwasm_std::{
    entry_point, to_json_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError, StdResult,
};
use cw2::set_contract_version;
use wavs_types::contracts::cosmwasm::{
    service_handler::{ServiceHandlerExecuteMessages, ServiceHandlerQueryMessages},
    service_manager::{ServiceManagerQueryMessages, WavsValidateResult},
};

use crate::state;
use cw_wavs_bls_api::service_handler::{
    BlsServiceHandlerQueryMessages, ExecuteMsg, InstantiateMsg, QueryMsg, TriggerMessageResponse,
};

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

    let service_manager = deps
        .api
        .addr_validate(&msg.service_manager)
        .map_err(|_| StdError::msg("Invalid service manager address"))?;

    state::SERVICE_MANAGER.save(deps.storage, &service_manager)?;

    Ok(Response::default().add_attribute("service_manager", service_manager))
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
            ServiceHandlerExecuteMessages::WavsHandleSignedEnvelope {
                envelope,
                signature_data,
            } => {
                let service_manager = state::SERVICE_MANAGER.load(deps.storage)?;

                deps.querier
                    .query_wasm_smart::<WavsValidateResult>(
                        service_manager,
                        &ServiceManagerQueryMessages::WavsValidate {
                            envelope: envelope.clone(),
                            signature_data: signature_data.clone(),
                        },
                    )?
                    .into_std()?;

                let decoded = state::save_envelope(deps.storage, envelope, signature_data)?;

                Ok(Response::new()
                    .add_attribute("method", "wavs_handle_signed_envelope")
                    .add_attribute("event_id", const_hex::encode(decoded.eventId.as_slice())))
            }
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Wavs(wavs) => match wavs {
            ServiceHandlerQueryMessages::WavsServiceManager {} => {
                to_json_binary(&state::SERVICE_MANAGER.load(deps.storage)?)
            }
        },
        QueryMsg::Bls(msg) => match msg {
            BlsServiceHandlerQueryMessages::TriggerValidated { trigger_id } => {
                to_json_binary(&state::TRIGGER_MESSAGE.has(deps.storage, trigger_id))
            }
            BlsServiceHandlerQueryMessages::TriggerMessage { trigger_id } => {
                let message = state::TRIGGER_MESSAGE
                    .may_load(deps.storage, trigger_id)?
                    .ok_or_else(|| {
                        StdError::msg(format!("no message for trigger_id {trigger_id}"))
                    })?;
                to_json_binary(&TriggerMessageResponse { message })
            }
            BlsServiceHandlerQueryMessages::SignatureData { trigger_id } => {
                let sig = state::SIGNATURE_DATA
                    .may_load(deps.storage, trigger_id)?
                    .ok_or_else(|| {
                        StdError::msg(format!("no signature data for trigger_id {trigger_id}"))
                    })?;
                to_json_binary(&sig)
            }
        },
    }
}
