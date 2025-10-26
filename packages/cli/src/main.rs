mod command;
mod context;
use utils::faucet;

use layer_climb_cli::command::WalletCommand;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    command::{
        contract::handle_contract_log,
        wallet::{handle_wallet_generate_env, handle_wallet_generate_single, handle_wallet_log},
        Command, RegistryCommand, ServiceHandlerCommand, ServiceManagerCommand,
    },
    context::CliContext,
};

#[tokio::main]
async fn main() {
    // Install rustls crypto provider before any TLS operations
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // setup tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false),
        )
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .unwrap();

    let mut ctx = CliContext::new().await;

    match ctx.command.clone() {
        Command::GenerateEnv { operators, .. } => {
            handle_wallet_generate_env(&mut ctx, operators).await;
        }
        Command::Wallet { command, .. } => match command {
            WalletCommand::Create => {
                handle_wallet_generate_single(&mut ctx).await;
            }
            _ => {
                command
                    .run(
                        ctx.climb_command_any_client().await.unwrap(),
                        &mut ctx.rng,
                        handle_wallet_log,
                    )
                    .await
                    .unwrap();
            }
        },
        Command::Contract { command, .. } => {
            command
                .run(
                    ctx.climb_command_any_client().await.unwrap(),
                    handle_contract_log,
                )
                .await
                .unwrap();
        }

        Command::ServiceManager { command } => match command {
            ServiceManagerCommand::Upload {
                wasm_directory,
                contract_kind,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();
                let wasm_path = contract_kind.wasm_path(&wasm_directory);
                let wasm_bytes = tokio::fs::read(&wasm_path)
                    .await
                    .unwrap_or_else(|_| panic!("Failed to read wasm file at {wasm_path}"));
                let (code_id, tx_resp) =
                    client.contract_upload_file(wasm_bytes, None).await.unwrap();

                println!("Uploaded {contract_kind} service manager");
                println!("Code ID: {code_id}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }
            ServiceManagerCommand::InstantiateMock { code_id, args: _ } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "Mock Service Manager",
                        &cw_wavs_mock_api::service_manager::InstantiateMsg {},
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();

                println!("Mock Service Manager instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }
            ServiceManagerCommand::InstantiateEcdsa { code_id, args: _ } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "ECSDA Service Manager",
                        &cw_wavs_ecdsa_api::service_manager::InstantiateMsg {},
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();

                println!("ECDSA Service Manager instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }
            ServiceManagerCommand::InstantiateBls { code_id, args: _ } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "BLS Service Manager",
                        &cw_wavs_bls_api::service_manager::InstantiateMsg {},
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();

                println!("BLS Service Manager instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }
            ServiceManagerCommand::InstantiateMirror {
                code_id,
                owner,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "Mirror Service Manager",
                        &cw_wavs_mirror_api::service_manager::InstantiateMsg { owner },
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();

                println!("Mirror Service Manager instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }

            ServiceManagerCommand::SetServiceUri {
                uri,
                address,
                args: _,
            } => {
                let client = ctx.wavs_service_manager_executor(&address).await.unwrap();
                let resp = client.set_service_uri(uri.to_string()).await.unwrap();
                println!(
                    "Set service URI TX hash: {}",
                    resp.unchecked_into_tx_response().txhash
                );
            }
            ServiceManagerCommand::GetServiceUri { address, args: _ } => {
                let client = ctx.wavs_service_manager_querier(&address).await.unwrap();
                let uri = client.get_service_uri().await.unwrap();
                println!("Service URI: {uri}");
            }
        },

        Command::ServiceHandler { command } => match command {
            ServiceHandlerCommand::Upload {
                wasm_directory,
                contract_kind,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();
                let wasm_path = contract_kind.wasm_path(&wasm_directory);
                let wasm_bytes = tokio::fs::read(&wasm_path)
                    .await
                    .unwrap_or_else(|_| panic!("Failed to read wasm file at {wasm_path}"));
                let (code_id, tx_resp) =
                    client.contract_upload_file(wasm_bytes, None).await.unwrap();

                println!("Uploaded {contract_kind} service handler");
                println!("Code ID: {code_id}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }
            ServiceHandlerCommand::InstantiateMock {
                code_id,
                service_manager,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "Mock Service Handler",
                        &cw_wavs_mock_api::service_handler::InstantiateMsg { service_manager },
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();
                println!("Mock Service Handler instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }
            ServiceHandlerCommand::InstantiateEcdsa {
                code_id,
                service_manager,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "ECDSA Service Handler",
                        &cw_wavs_ecdsa_api::service_handler::InstantiateMsg { service_manager },
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();
                println!("ECDSA Service Handler instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }
            ServiceHandlerCommand::InstantiateBls {
                code_id,
                service_manager,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "BLS Service Handler",
                        &cw_wavs_bls_api::service_handler::InstantiateMsg { service_manager },
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();
                println!("BLS Service Handler instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }
            ServiceHandlerCommand::InstantiateMirror {
                code_id,
                service_manager,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "Mirror Service Handler",
                        &cw_wavs_mirror_api::service_handler::InstantiateMsg { service_manager },
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();
                println!("Mirror Service Handler instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }

            ServiceHandlerCommand::GetManager { address, args: _ } => {
                let client = ctx.wavs_service_handler_querier(&address).await.unwrap();
                let manager = client.get_manager_address().await.unwrap();
                println!("Service Manager: {manager}");
            }
        },

        Command::Registry { command } => match command {
            RegistryCommand::Upload {
                wasm_directory,
                contract_kind,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();
                let wasm_path = contract_kind.wasm_path(&wasm_directory);
                let wasm_bytes = tokio::fs::read(&wasm_path)
                    .await
                    .unwrap_or_else(|_| panic!("Failed to read wasm file at {wasm_path}"));
                let (code_id, tx_resp) =
                    client.contract_upload_file(wasm_bytes, None).await.unwrap();

                println!("Uploaded {contract_kind} registry");
                println!("Code ID: {code_id}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }
            RegistryCommand::InstantiateMirrorStake {
                code_id,
                service_manager_code_id,
                threshold_weight,
                strategy,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let service_manager_instantiate = cosmwasm_std::WasmMsg::Instantiate2 {
                    admin: Some(client.addr.to_string()),
                    code_id: service_manager_code_id,
                    msg: cosmwasm_std::to_json_binary(
                        &cw_wavs_mirror_api::service_manager::InstantiateMsg {
                            owner: client.addr.to_string(),
                        },
                    )
                    .unwrap(),
                    funds: vec![],
                    label: "Mirror Service Manager".to_string(),
                    salt: cosmwasm_std::Binary::from(b"service_manager"),
                };

                let strategies = strategy
                    .into_iter()
                    .map(|s| {
                        let (strategy, multiplier) = s.split_once('=').unwrap_or_else(|| {
                            panic!(
                                "Strategy must be in the format strategy=multiplier, got: {}",
                                s
                            )
                        });

                        let multiplier: cosmwasm_std::Uint256 =
                            multiplier.parse().unwrap_or_else(|_| {
                                panic!(
                                    "Multiplier must be a valid u128 integer, got: {}",
                                    multiplier
                                )
                            });
                        cw_wavs_mirror_api::stake_registry::StrategyParams {
                            strategy: strategy.to_string(),
                            multiplier,
                        }
                    })
                    .collect();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "Mirror Stake Registry",
                        &cw_wavs_mirror_api::stake_registry::InstantiateMsg {
                            service_manager_instantiate,
                            threshold_weight: threshold_weight.into(),
                            quorum: cw_wavs_mirror_api::stake_registry::QuorumConfig { strategies },
                        },
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();
                println!("Mirror Stake Registry instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash)
            }
        },

        Command::FaucetTap { addr, url, .. } => {
            let client = ctx.query_client().await.unwrap();
            let addr = match addr {
                Some(addr) => ctx.parse_address(&addr).await.unwrap(),
                None => ctx.wallet_addr().await.unwrap(),
            };
            let balance_before = client
                .balance(addr.clone(), None)
                .await
                .unwrap()
                .unwrap_or_default();
            faucet::tap(&addr, &client.chain_config.gas_denom, Some(&url))
                .await
                .unwrap();
            let balance_after = client
                .balance(addr.clone(), None)
                .await
                .unwrap()
                .unwrap_or_default();

            println!(
                "Tapped faucet for {addr} - balance before: {balance_before} balance after: {balance_after}"
            );
        }
    }
}
