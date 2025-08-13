use crate::bindings::wavs::worker::input::TriggerData;
use crate::bindings::{Guest, TriggerAction, WasmResponse};
use crate::error::EchoResult;

struct Component;

impl Guest for Component {
    fn run(trigger_action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        match trigger_action.data {
            TriggerData::CosmosContractEvent(_event) => Ok(None),
            TriggerData::Raw(raw) => handle_raw(raw).map_err(|e| e.to_string()),
            _ => Err("Unsupported trigger data".to_string()),
        }
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
