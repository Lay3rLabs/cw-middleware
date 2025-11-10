use alloy_sol_types::SolType;
use cosmwasm_std::{
    ensure, entry_point, to_json_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryResponse,
    Response, StdError, StdResult, Uint256, WasmMsg,
};
use cw_wavs_mirror_api::service_manager::MirrorServiceManagerQueryMessages;
use cw_wavs_mirror_api::update_with_id::IMirrorOperatorSyncHandler::UpdateWithId;
use cw_wavs_mirror_service_handler::state::{self};
use wavs_types::contracts::cosmwasm::service_manager::ServiceManagerQueryMessages;
use wavs_types::contracts::cosmwasm::{
    service_handler::ServiceHandlerExecuteMessages, service_manager::WavsValidateResult,
};

use cw_wavs_mirror_api::service_handler::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::state::LAST_TRIGGER_ID;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    cw_wavs_mirror_service_handler::entry::instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let mut msgs = vec![];

    match msg {
        ExecuteMsg::Wavs(msg) => {
            match msg {
                ServiceHandlerExecuteMessages::WavsHandleSignedEnvelope {
                    envelope,
                    signature_data,
                } => {
                    // Decode envelope
                    let decoded_envelope = envelope.decode()?;
                    let UpdateWithId { triggerId, thresholdWeight, operators, signingKeyAddresses, weights } = cw_wavs_mirror_api::update_with_id::IMirrorOperatorSyncHandler::UpdateWithId::abi_decode(&decoded_envelope.payload)?;

                    // Validate trigger id
                    if let Some(last_trigger_id) = LAST_TRIGGER_ID.may_load(deps.storage)? {
                        ensure!(
                            last_trigger_id < triggerId,
                            StdError::msg("Invalid trigger id")
                        );
                        LAST_TRIGGER_ID.save(deps.storage, &triggerId)?;
                    }

                    // Validate against service manager
                    let contract_addr = state::SERVICE_MANAGER.load(deps.storage)?;
                    deps.querier
                        .query_wasm_smart::<WavsValidateResult>(
                            &contract_addr,
                            &ServiceManagerQueryMessages::WavsValidate {
                                envelope: envelope.clone(),
                                signature_data: signature_data.clone(),
                            },
                        )?
                        .into_std()?;

                    // Perform sync
                    let stake_registry: String = deps.querier.query_wasm_smart(
                        contract_addr,
                        &MirrorServiceManagerQueryMessages::StakeRegistry {},
                    )?;

                    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: stake_registry.clone(),
                        msg: to_json_binary(&cw_wavs_mirror_api::stake_registry::ExecuteMsg::BatchSetOperatorDetails {
                                operators: operators.into_iter().map(Into::into).collect(),
                                signing_keys:signingKeyAddresses.into_iter().map(Into::into).collect(),
                                weights: weights.into_iter().map(|x| Uint256::from_be_bytes(x.to_be_bytes())).collect()
                            }
                        )?,
                        funds: vec![],
                    }));
                    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: stake_registry,
                        msg: to_json_binary(
                            &cw_wavs_mirror_api::stake_registry::ExecuteMsg::UpdateStakeThreshold {
                                threshold: Uint256::from_be_bytes(thresholdWeight.to_be_bytes()),
                            },
                        )?,
                        funds: vec![],
                    }));
                }
            }
        }
    };

    Ok(Response::new().add_messages(msgs))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    cw_wavs_mirror_service_handler::entry::query(deps, env, msg)
}
