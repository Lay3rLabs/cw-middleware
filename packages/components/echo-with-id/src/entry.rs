use mock_api::data_with_id::DataWithId;
use mock_api::trigger::PushMessageEvent;

use layer_climb::prelude::*;

use crate::bindings::wavs::worker::helpers::LogLevel;
use crate::bindings::wavs::worker::input::TriggerData;
use crate::bindings::{host, Guest, TriggerAction, WasmResponse};
use crate::error::EchoResult;

struct Component;

impl Guest for Component {
    fn run(trigger_action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        Err("WTF!".to_string())
        // let res = inner(trigger_action);

        // host::log(LogLevel::Warn, &format!("Echo response: {:?}", res));

        // res
    }
}

fn inner(trigger_action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
    match trigger_action.data {
        TriggerData::CosmosContractEvent(data) => {

            let cosmos_event = cosmwasm_std::Event::new(data.event.ty).add_attributes(data.event.attributes);

            let event = PushMessageEvent::try_from(&cosmos_event).map_err(|e| e.to_string())?;

            let chain_config = host::get_cosmos_chain_config(&data.chain_name).ok_or_else(|| {
                format!("No chain config found for {}", data.chain_name)
            })?;


            let message = wstd::runtime::block_on(async move {
                let client = QueryClient::new(ChainConfig { 
                    chain_id: chain_config.chain_id.parse().map_err(|_| "Invalid chain ID")?, 
                    rpc_endpoint: chain_config.rpc_endpoint, 
                    grpc_endpoint: chain_config.grpc_endpoint, 
                    grpc_web_endpoint: None, 
                    gas_price: chain_config.gas_price, 
                    gas_denom: chain_config.gas_denom, 
                    address_kind: AddrKind::Cosmos { prefix: chain_config.bech32_prefix },
                }, None).await.map_err(|e| e.to_string())?;

                // TODO - use utils extension

                let address = Address::Cosmos { 
                    bech32_addr: data.contract_address.bech32_addr,
                    prefix_len: data.contract_address.prefix_len.try_into().map_err(|_| "Invalid prefix length")?
                };

                let message:String = client.contract_smart(&address, &mock_api::trigger::QueryMsg::TriggerMessage { trigger_id: event.trigger_id }).await
                    .map_err(|e| e.to_string())?;

                Result::<String, String>::Ok(message)
            })?;

            Ok(Some(WasmResponse {
                payload: DataWithId {
                    trigger_id: event.trigger_id,
                    data: message.into_bytes().into()
                }.to_bytes().map_err(|e| e.to_string())?, 
                ordering: None,
            }))
        },
        TriggerData::Raw(raw) => handle_raw(raw).map_err(|e| e.to_string()),
        _ => Err("Unsupported trigger data".to_string()),
    }
}

fn handle_raw(raw: Vec<u8>) -> EchoResult<Option<WasmResponse>> {
    let input = String::from_utf8(raw)?;
    let response = format!("Echo: {input}");

    Ok(Some(WasmResponse {
        payload: response.into_bytes(),
        ordering: None,
    }))
}

crate::bindings::export!(Component with_types_in crate::bindings);
