use std::str::FromStr;

use wavs_types::ChainKey;
use wavs_wasi_utils::evm::alloy_primitives;

use crate::bindings::{
    host,
    wavs::{
        aggregator::aggregator::{CosmosSubmitAction, EvmAddress, EvmSubmitAction, SubmitAction},
        types::chain::CosmosAddress,
    },
    AggregatorAction, AnyTxHash, Guest, Packet,
};

mod bindings;
struct Component;

impl Guest for Component {
    fn process_packet(_pkt: Packet) -> Result<Vec<AggregatorAction>, String> {
        let chain = host::config_var("chain").ok_or("chain config variable is required")?;
        let chain =
            AnyChainKey::from_host(&chain).ok_or(format!("no chain config for {}", chain))?;
        let service_handler_str = host::config_var("service_handler")
            .ok_or("service_handler config variable is required")?;

        let submit_action = match chain {
            AnyChainKey::Evm(chain) => SubmitAction::Evm(EvmSubmitAction {
                chain: chain.to_string(),
                address: EvmAddress {
                    raw_bytes: alloy_primitives::Address::from_str(&service_handler_str)
                        .map_err(|e| e.to_string())?
                        .to_vec(),
                },
                gas_price: None,
            }),
            AnyChainKey::Cosmos(chain) => {
                let address = layer_climb::prelude::CosmosAddr::new_str(&service_handler_str, None)
                    .map_err(|e| e.to_string())?;

                SubmitAction::Cosmos(CosmosSubmitAction {
                    chain: chain.to_string(),
                    address: CosmosAddress {
                        bech32_addr: address.to_string(),
                        prefix_len: address.prefix().len() as u32,
                    },
                    gas_price: None,
                })
            }
        };

        Ok(vec![AggregatorAction::Submit(submit_action)])
    }

    fn handle_timer_callback(_packet: Packet) -> Result<Vec<AggregatorAction>, String> {
        Err("Not implemented yet".to_string())
    }

    fn handle_submit_callback(
        _packet: Packet,
        tx_result: Result<AnyTxHash, String>,
    ) -> Result<(), String> {
        match tx_result {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }
}

enum AnyChainKey {
    Evm(ChainKey),
    Cosmos(ChainKey),
}

impl AnyChainKey {
    pub fn from_host(chain: &str) -> Option<Self> {
        match host::get_evm_chain_config(chain) {
            Some(_) => Some(AnyChainKey::Evm(chain.parse().ok()?)),
            None => match host::get_cosmos_chain_config(chain) {
                Some(_) => Some(AnyChainKey::Cosmos(chain.parse().ok()?)),
                None => None,
            },
        }
    }
}

crate::bindings::export!(Component with_types_in crate::bindings);
