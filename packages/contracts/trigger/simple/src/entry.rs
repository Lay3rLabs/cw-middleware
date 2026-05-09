use cosmwasm_std::{
    entry_point, to_json_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError, StdResult, Uint64,
};
use cw2::set_contract_version;

use crate::state;
use cw_wavs_trigger_api::simple::{ExecuteMsg, InstantiateMsg, PushMessageEvent, QueryMsg};

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

    // Audit M-1: optional pusher allowlist. None = legacy public-bus
    // behavior. Validation surfaces invalid addresses at instantiate.
    let allowed_pushers = match msg.allowed_pushers {
        Some(list) => {
            let mut validated = Vec::with_capacity(list.len());
            for s in list {
                validated.push(deps.api.addr_validate(&s)?);
            }
            Some(validated)
        }
        None => None,
    };
    state::ALLOWED_PUSHERS.save(deps.storage, &allowed_pushers)?;

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
        ExecuteMsg::Push { data } => {
            // Audit M-1: enforce pusher allowlist if configured. The legacy
            // public-bus behavior is preserved when ALLOWED_PUSHERS is None.
            if let Some(allowlist) = state::ALLOWED_PUSHERS.may_load(deps.storage)?.flatten() {
                if !allowlist.contains(&info.sender) {
                    return Err(StdError::msg("Unauthorized: sender not in allowed_pushers"));
                }
            }

            let trigger_id: u64 = state::TRIGGER_MESSAGE_COUNT
                .may_load(deps.storage)?
                .unwrap_or_default()
                + 1;

            state::TRIGGER_MESSAGE_COUNT.save(deps.storage, &trigger_id)?;

            let trigger_id = Uint64::new(trigger_id);

            state::TRIGGER_MESSAGES.save(deps.storage, trigger_id, &data)?;

            Ok(Response::new().add_event(PushMessageEvent { trigger_id, data }))
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::TriggerMessage { trigger_id } => {
            to_json_binary(&state::TRIGGER_MESSAGES.load(deps.storage, trigger_id)?)
        }
    }
}
