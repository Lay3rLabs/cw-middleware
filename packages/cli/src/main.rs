mod command;
mod context;
use utils::faucet;

use layer_climb_cli::command::WalletCommand;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    command::{
        contract::handle_contract_log,
        wallet::{handle_wallet_generate_env, handle_wallet_generate_single, handle_wallet_log},
        Command, ContractArgs, ContractKind, ServiceHandlerArgs, ServiceHandlerCommand,
        ServiceManagerArgs, ServiceManagerCommand, WalletArgs,
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

    match ctx.args.command.clone() {
        Command::GenerateEnv { operators } => {
            handle_wallet_generate_env(&mut ctx, operators).await;
        }
        Command::Wallet(WalletArgs { command }) => match command {
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
        Command::Contract(ContractArgs { command }) => {
            command
                .run(
                    ctx.climb_command_any_client().await.unwrap(),
                    handle_contract_log,
                )
                .await
                .unwrap();
        }

        Command::ServiceManager(ServiceManagerArgs { command }) => match command {
            ServiceManagerCommand::Deploy {
                code_id,
                contract_kind,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, _) = match contract_kind {
                    ContractKind::Mock => client
                        .contract_instantiate(
                            None,
                            code_id,
                            "Mock Service Manager",
                            &cw_wavs_mock_api::service_manager::InstantiateMsg {},
                            Vec::new(),
                            None,
                        )
                        .await
                        .unwrap(),
                    ContractKind::Ecdsa => client
                        .contract_instantiate(
                            None,
                            code_id,
                            "Ecdsa Service Manager",
                            &cw_wavs_ecdsa_api::service_manager::InstantiateMsg {},
                            Vec::new(),
                            None,
                        )
                        .await
                        .unwrap(),
                    ContractKind::Bls => client
                        .contract_instantiate(
                            None,
                            code_id,
                            "Bls Service Manager",
                            &cw_wavs_bls_api::service_manager::InstantiateMsg {},
                            Vec::new(),
                            None,
                        )
                        .await
                        .unwrap(),
                };

                println!("Service Manager deployed at: {address}");
            }
            ServiceManagerCommand::SetServiceUri { uri, address } => {
                let client = ctx.wavs_service_manager_executor(&address).await.unwrap();
                let resp = client.set_service_uri(uri.to_string()).await.unwrap();
                println!(
                    "Set service URI TX hash: {}",
                    resp.unchecked_into_tx_response().txhash
                );
            }
            ServiceManagerCommand::GetServiceUri { address } => {
                let client = ctx.wavs_service_manager_querier(&address).await.unwrap();
                let uri = client.get_service_uri().await.unwrap();
                println!("Service URI: {uri}");
            }
        },

        Command::ServiceHandler(ServiceHandlerArgs { command }) => match command {
            ServiceHandlerCommand::Deploy {
                code_id,
                service_manager,
                contract_kind,
            } => {
                let client = ctx.signing_client().await.unwrap();

                let (address, _) = match contract_kind {
                    ContractKind::Mock => client
                        .contract_instantiate(
                            None,
                            code_id,
                            "Mock Service Handler",
                            &cw_wavs_mock_api::service_handler::InstantiateMsg { service_manager },
                            Vec::new(),
                            None,
                        )
                        .await
                        .unwrap(),
                    ContractKind::Ecdsa => client
                        .contract_instantiate(
                            None,
                            code_id,
                            "Ecdsa Service Handler",
                            &cw_wavs_ecdsa_api::service_handler::InstantiateMsg { service_manager },
                            Vec::new(),
                            None,
                        )
                        .await
                        .unwrap(),
                    ContractKind::Bls => client
                        .contract_instantiate(
                            None,
                            code_id,
                            "Bls Service Handler",
                            &cw_wavs_bls_api::service_handler::InstantiateMsg { service_manager },
                            Vec::new(),
                            None,
                        )
                        .await
                        .unwrap(),
                };
                println!("Service Handler deployed at: {address}");
            }
            ServiceHandlerCommand::GetManager { address } => {
                let client = ctx.wavs_service_handler_querier(&address).await.unwrap();
                let manager = client.get_manager_address().await.unwrap();
                println!("Service Manager: {manager}");
            }
        },

        Command::FaucetTap { addr, url } => {
            let client = ctx.query_client().await.unwrap();
            let addr = match addr {
                Some(addr) => ctx.parse_address(&addr).unwrap(),
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
