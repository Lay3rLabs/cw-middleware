mod command;
mod context;
mod output;
use utils::faucet;

use layer_climb_cli::command::WalletCommand;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    command::{
        contract::handle_contract_log,
        wallet::{handle_wallet_generate_env, handle_wallet_generate_single, handle_wallet_log},
        Command, RegistryCommand, RegistryContractKind, ServiceHandlerCommand,
        ServiceHandlerContractKind, ServiceManagerCommand, ServiceManagerContractKind,
    },
    context::CliContext,
};
use cosmwasm_std::Uint256;
use cw_wavs_sdk::client::WavsExecutor;

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
                println!("Tx Hash: {}", tx_resp.txhash);

                ctx.output
                    .write(output::OutputData::ServiceManagerUpload {
                        contract_kind,
                        code_id,
                        tx_hash: tx_resp.txhash,
                    })
                    .await
                    .unwrap();
            }
            ServiceManagerCommand::InstantiateEcdsa {
                code_id,
                owner,
                admin,
                quorum_numerator,
                quorum_denominator,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "ECDSA Service Manager",
                        &cw_wavs_ecdsa_api::service_manager::InstantiateMsg {
                            owner,
                            admin,
                            quorum_numerator: quorum_numerator
                                .map(|s| s.parse().expect("invalid numerator")),
                            quorum_denominator: quorum_denominator
                                .map(|s| s.parse().expect("invalid denominator")),
                        },
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();

                println!("ECDSA Service Manager instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash);

                ctx.output
                    .write(output::OutputData::ServiceManagerInstantiate {
                        contract_kind: ServiceManagerContractKind::Ecdsa,
                        address: address.to_string(),
                        tx_hash: tx_resp.txhash,
                    })
                    .await
                    .unwrap();
            }
            ServiceManagerCommand::InstantiateBls {
                code_id,
                owner,
                admin,
                quorum_numerator,
                quorum_denominator,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "BLS Service Manager",
                        &cw_wavs_bls_api::service_manager::InstantiateMsg {
                            owner,
                            admin,
                            quorum_numerator: quorum_numerator
                                .map(|s| s.parse().expect("invalid numerator")),
                            quorum_denominator: quorum_denominator
                                .map(|s| s.parse().expect("invalid denominator")),
                        },
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();

                println!("BLS Service Manager instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash);

                ctx.output
                    .write(output::OutputData::ServiceManagerInstantiate {
                        contract_kind: ServiceManagerContractKind::Bls,
                        address: address.to_string(),
                        tx_hash: tx_resp.txhash,
                    })
                    .await
                    .unwrap();
            }
            ServiceManagerCommand::InstantiateMirror {
                code_id,
                admin,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, tx_resp) = client
                    .contract_instantiate(
                        None,
                        code_id,
                        "Mirror Service Manager",
                        &cw_wavs_mirror_api::service_manager::InstantiateMsg { admin },
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();

                println!("Mirror Service Manager instantiated at: {address}");
                println!("Tx Hash: {}", tx_resp.txhash);

                ctx.output
                    .write(output::OutputData::ServiceManagerInstantiate {
                        contract_kind: ServiceManagerContractKind::Mirror,
                        address: address.to_string(),
                        tx_hash: tx_resp.txhash,
                    })
                    .await
                    .unwrap();
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
            ServiceManagerCommand::SetQuorumThreshold {
                numerator,
                denominator,
                address,
                args: _,
            } => {
                let client = ctx.wavs_service_manager_executor(&address).await.unwrap();
                let numerator: Uint256 = numerator.parse().expect("Invalid numerator value");
                let denominator: Uint256 = denominator.parse().expect("Invalid denominator value");
                let resp = client
                    .set_quorum_threshold(numerator, denominator)
                    .await
                    .unwrap();
                println!(
                    "Set quorum threshold TX hash: {}",
                    resp.unchecked_into_tx_response().txhash
                );
            }
            ServiceManagerCommand::SetMirrorAdmin {
                address,
                new_admin,
                args: _,
            } => {
                let client = ctx.signing_client().await.unwrap();
                let address = ctx.parse_address(&address).await.unwrap();
                let resp = client
                    .contract_execute(
                        &address,
                        &cw_wavs_mirror_api::service_manager::ExecuteMsg::SetAdmin { new_admin },
                        Vec::new(),
                        None,
                    )
                    .await
                    .unwrap();
                println!("Mirror SetAdmin TX hash: {}", resp.txhash);
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
                println!("Tx Hash: {}", tx_resp.txhash);

                ctx.output
                    .write(output::OutputData::ServiceHandlerUpload {
                        contract_kind,
                        code_id,
                        tx_hash: tx_resp.txhash,
                    })
                    .await
                    .unwrap();
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
                println!("Tx Hash: {}", tx_resp.txhash);

                ctx.output
                    .write(output::OutputData::ServiceHandlerInstantiate {
                        contract_kind: ServiceHandlerContractKind::Ecdsa,
                        address: address.to_string(),
                        tx_hash: tx_resp.txhash,
                    })
                    .await
                    .unwrap();
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
                println!("Tx Hash: {}", tx_resp.txhash);

                ctx.output
                    .write(output::OutputData::ServiceHandlerInstantiate {
                        contract_kind: ServiceHandlerContractKind::Bls,
                        address: address.to_string(),
                        tx_hash: tx_resp.txhash,
                    })
                    .await
                    .unwrap();
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
                println!("Tx Hash: {}", tx_resp.txhash);

                ctx.output
                    .write(output::OutputData::ServiceHandlerInstantiate {
                        contract_kind: ServiceHandlerContractKind::Mirror,
                        address: address.to_string(),
                        tx_hash: tx_resp.txhash,
                    })
                    .await
                    .unwrap();
            }

            ServiceHandlerCommand::GetManager { address, args: _ } => {
                let client = ctx.wavs_service_handler_querier(&address).await.unwrap();
                let manager = client.get_manager_address().await.unwrap();
                println!("Service Manager: {manager}");
            }
        },

        Command::Registry { command } => {
            match command {
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
                    println!("Tx Hash: {}", tx_resp.txhash);

                    ctx.output
                        .write(output::OutputData::RegistryUpload {
                            contract_kind,
                            code_id,
                            tx_hash: tx_resp.txhash,
                        })
                        .await
                        .unwrap();
                }
                RegistryCommand::InstantiateMirrorStake {
                    code_id,
                    service_manager_code_id,
                    args: _,
                } => {
                    let client = ctx.signing_client().await.unwrap();

                    let service_manager_instantiate = cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some(client.addr.to_string()),
                        code_id: service_manager_code_id,
                        msg: cosmwasm_std::to_json_binary(
                            &cw_wavs_mirror_api::service_manager::InstantiateMsg {
                                admin: client.addr.to_string(),
                            },
                        )
                        .unwrap(),
                        funds: vec![],
                        label: "Mirror Service Manager".to_string(),
                        salt: cosmwasm_std::Binary::from(b"service_manager"),
                    };

                    let (registry_address, tx_resp) = client
                        .contract_instantiate(
                            None,
                            code_id,
                            "Mirror Stake Registry",
                            &cw_wavs_mirror_api::stake_registry::InstantiateMsg {
                                service_manager_instantiate,
                            },
                            Vec::new(),
                            None,
                        )
                        .await
                        .unwrap();

                    let service_manager_address: cosmwasm_std::Addr = client
                        .querier
                        .contract_smart(
                            &registry_address,
                            &cw_wavs_mirror_api::stake_registry::QueryMsg::GetServiceManager {},
                        )
                        .await
                        .unwrap();

                    println!("Mirror Stake Registry instantiated at: {registry_address}");
                    println!("Service Manager instantiated at: {service_manager_address}");
                    println!("Tx Hash: {}", tx_resp.txhash);

                    ctx.output
                        .write(output::OutputData::RegistryInstantiate {
                            contract_kind: RegistryContractKind::MirrorStake,
                            registry_address: registry_address.to_string(),
                            service_manager_address: service_manager_address.to_string(),
                            tx_hash: tx_resp.txhash,
                        })
                        .await
                        .unwrap();
                }
                RegistryCommand::GetServiceManager {
                    address,
                    contract_kind,
                    args: _,
                } => {
                    let manager: cosmwasm_std::Addr = match contract_kind {
                        RegistryContractKind::MirrorStake => {
                            let client = ctx.query_client().await.unwrap();
                            let address = client.chain_config.parse_address(&address).unwrap();
                            client.contract_smart(&address, &cw_wavs_mirror_api::stake_registry::QueryMsg::GetServiceManager {  }).await.unwrap()
                        }
                    };
                    println!("Service Manager: {manager}");
                }
                RegistryCommand::SetOperatorSigningKey {
                    address,
                    operator,
                    signing_key,
                    weight,
                    args: _,
                } => {
                    // Parse input
                    let client = ctx.signing_client().await.unwrap();
                    let address = ctx.parse_address(&address).await.unwrap();
                    let operator_weight: Uint256 = weight.parse().expect("Invalid weight value");

                    // Create stake registry executor from client
                    let stake_registry =
                        cw_wavs_sdk::contract_kinds::mirror::MirrorStakeRegistryExecutor::new(
                            WavsExecutor::Climb(client),
                            address
                                .try_into()
                                .expect("Stake registry address is not a cosmos address"),
                        );

                    // Execute
                    let tx_resp = stake_registry
                        .set_operator_details(operator, signing_key, operator_weight)
                        .await
                        .unwrap();

                    println!("Operator signing key set successfully!");
                    println!(
                        "Transaction Hash: {}",
                        tx_resp.unchecked_into_tx_response().txhash
                    );
                }
                RegistryCommand::TransferOwnership {
                    address,
                    new_owner,
                    args: _,
                } => {
                    let client = ctx.signing_client().await.unwrap();
                    let address = ctx.parse_address(&address).await.unwrap();
                    let resp = client
                        .contract_execute(
                            &address,
                            &cw_wavs_mirror_api::stake_registry::ExecuteMsg::TransferOwnership {
                                new_owner,
                            },
                            Vec::new(),
                            None,
                        )
                        .await
                        .unwrap();
                    println!(
                        "Mirror stake-registry TransferOwnership TX hash: {}",
                        resp.txhash
                    );
                }
            }
        }

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
