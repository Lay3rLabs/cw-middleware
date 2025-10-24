use cw_wavs_mock_api::message_with_id::MessageWithId;
use cw_wavs_sdk::contract_kinds::trigger::SimpleTriggerQuerier;
use cw_wavs_trigger_api::simple::PushMessageEvent;

use layer_climb::prelude::*;

use crate::bindings::wavs::operator::input::TriggerData;
use crate::bindings::wavs::types::core::LogLevel;
use crate::bindings::{host, Guest, TriggerAction, WasmResponse};
use crate::error::EchoResult;

struct Component;

impl Guest for Component {
    fn run(trigger_action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let res = inner(trigger_action);

        host::log(LogLevel::Warn, &format!("Echo response: {res:?}"));

        res
    }
}

fn inner(trigger_action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
    match trigger_action.data {
        TriggerData::CosmosContractEvent(data) => {
            let cosmos_event =
                cosmwasm_std::Event::new(data.event.ty).add_attributes(data.event.attributes);

            let event = PushMessageEvent::try_from(&cosmos_event).map_err(|e| e.to_string())?;

            let chain_config = host::get_cosmos_chain_config(&data.chain)
                .ok_or_else(|| format!("No chain config found for {}", data.chain))?;

            let message = wstd::runtime::block_on(async move {
                let client = QueryClient::new(
                    ChainConfig {
                        chain_id: chain_config
                            .chain_id
                            .parse()
                            .map_err(|_| "Invalid chain ID")?,
                        rpc_endpoint: chain_config.rpc_endpoint,
                        grpc_endpoint: chain_config.grpc_endpoint,
                        grpc_web_endpoint: None,
                        gas_price: chain_config.gas_price,
                        gas_denom: chain_config.gas_denom,
                        address_kind: AddrKind::Cosmos {
                            prefix: chain_config.bech32_prefix,
                        },
                    },
                    None,
                )
                .await
                .map_err(|e| e.to_string())?;

                let address = CosmosAddr::new_str(&data.contract_address.bech32_addr, None)
                    .map_err(|_| "Invalid prefix length")?;

                let trigger = SimpleTriggerQuerier::new(
                    client.into(),
                    cosmwasm_std::Addr::try_from(address).map_err(|e| e.to_string())?,
                );

                let message = trigger
                    .get_trigger_message(event.trigger_id)
                    .await
                    .map_err(|e| e.to_string())?;

                Result::<String, String>::Ok(message)
            })?;

            Ok(Some(WasmResponse {
                payload: MessageWithId {
                    trigger_id: event.trigger_id,
                    message,
                }
                .to_bytes()
                .map_err(|e| e.to_string())?,
                ordering: None,
            }))
        }
        TriggerData::Raw(raw) => handle_raw(raw).map_err(|e| e.to_string()),
        _ => Err("Unsupported trigger data".to_string()),
    }
}

pub fn handle_raw(raw: Vec<u8>) -> EchoResult<Option<WasmResponse>> {
    let input = String::from_utf8(raw)?;

    Ok(Some(WasmResponse {
        payload: input.into_bytes(),
        ordering: None,
    }))
}

crate::bindings::export!(Component with_types_in crate::bindings);
