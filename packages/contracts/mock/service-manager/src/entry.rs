use cosmwasm_std::{
    entry_point, to_json_binary, Deps, DepsMut, Empty, Env, MessageInfo, QueryResponse, Response,
    StdResult, Uint256,
};
use cw2::set_contract_version;
use wavs_types::contracts::cosmwasm::service_manager::{
    error::WavsValidateError, event::WavsServiceUriUpdatedEvent, ServiceManagerExecuteMessages,
    ServiceManagerQueryMessages, WavsValidateResult,
};

use crate::state::{self, QUORUM_DENOMINATOR, QUORUM_NUMERATOR};
use cw_wavs_mock_api::service_manager::{ExecuteMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Set default quorum configuration (2/3)
    let default_numerator = Uint256::from(2u128);
    let default_denominator = Uint256::from(3u128);
    QUORUM_NUMERATOR.save(deps.storage, &default_numerator)?;
    QUORUM_DENOMINATOR.save(deps.storage, &default_denominator)?;

    Ok(Response::default()
        .add_attribute("quorum_numerator", default_numerator.to_string())
        .add_attribute("quorum_denominator", default_denominator.to_string()))
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
            ServiceManagerExecuteMessages::WavsSetQuorumThreshold {
                numerator,
                denominator,
            } => {
                // For Mock service manager, we'll allow any sender to set quorum threshold for simplicity
                // In a real implementation, this should be restricted to admin/owner

                // Validate quorum parameters
                if numerator.is_zero() || denominator.is_zero() || numerator > denominator {
                    return Err(cosmwasm_std::StdError::msg("Invalid quorum parameters: numerator must be > 0, denominator must be > 0, and numerator must be <= denominator"));
                }

                QUORUM_NUMERATOR.save(deps.storage, &numerator)?;
                QUORUM_DENOMINATOR.save(deps.storage, &denominator)?;

                Ok(Response::new()
                    .add_attribute("method", "wavs_set_quorum_threshold")
                    .add_attribute("numerator", numerator.to_string())
                    .add_attribute("denominator", denominator.to_string()))
            }
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
            state::OPERATOR_SIGNING_KEY_ADDRS.save(deps.storage, &operator, &signing_key)?;
            state::SIGNING_KEY_OPERATOR_ADDRS.save(deps.storage, &signing_key, &operator)?;
            state::OPERATOR_WEIGHTS.save(deps.storage, &operator, &weight)?;
            Ok(Response::default())
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
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
                // TODO: query stake registry etc.
                to_json_binary(&state::OPERATOR_WEIGHTS.load(deps.storage, &operator_address)?)
            }
            ServiceManagerQueryMessages::WavsValidate {
                envelope: _,
                signature_data,
            } => {
                // TODO: real validation logic
                for signer in &signature_data.signers {
                    if !state::SIGNING_KEY_OPERATOR_ADDRS.has(deps.storage, signer) {
                        return to_json_binary(&WavsValidateResult::Err(
                            WavsValidateError::InvalidSignature(format!(
                                "Signer address {} not recognized",
                                signer
                            )),
                        ));
                    };
                }
                to_json_binary(&WavsValidateResult::Ok)
            }
            ServiceManagerQueryMessages::WavsServiceUri {} => {
                to_json_binary(&state::SERVICE_URI.load(deps.storage)?)
            }
            ServiceManagerQueryMessages::WavsLatestOperatorForSigningKey { signing_key_addr } => {
                to_json_binary(
                    &state::SIGNING_KEY_OPERATOR_ADDRS.may_load(deps.storage, &signing_key_addr)?,
                )
            }
        },
    }
}
