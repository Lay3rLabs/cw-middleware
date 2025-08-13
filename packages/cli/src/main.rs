mod command;
mod context;
use utils::prelude::*;

use layer_climb_cli::command::WalletCommand;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    command::{
        contract::handle_contract_log,
        wallet::{handle_wallet_generate_env, handle_wallet_generate_single, handle_wallet_log},
        Command, ContractArgs, ServiceHandlerArgs, ServiceHandlerCommand, ServiceManagerArgs,
        ServiceManagerCommand, WalletArgs,
    },
    context::CliContext,
};

#[tokio::main]
async fn main() {
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

    match &ctx.args.command {
        Command::GenerateEnv => {
            handle_wallet_generate_env(&mut ctx).await;
        }
        Command::Wallet(WalletArgs { command }) => match command {
            WalletCommand::Create => {
                handle_wallet_generate_single(&mut ctx).await;
            }
            _ => {
                command
                    .run(
                        ctx.any_client().await.unwrap(),
                        &mut ctx.rng,
                        handle_wallet_log,
                    )
                    .await
                    .unwrap();
            }
        },
        Command::Contract(ContractArgs { command }) => {
            command
                .run(ctx.any_client().await.unwrap(), handle_contract_log)
                .await
                .unwrap();
        }
        Command::ServiceManager(ServiceManagerArgs { command, address }) => match command {
            ServiceManagerCommand::SetServiceUri { uri } => {
                let client = ctx
                    .wavs_service_manager_signing_client(address)
                    .await
                    .unwrap();
                let resp = client.set_service_uri(uri.to_string()).await.unwrap();
                println!("Set service URI TX hash: {}", resp.txhash);
            }
            ServiceManagerCommand::GetServiceUri => {
                let client = ctx
                    .wavs_service_manager_query_client(address)
                    .await
                    .unwrap();
                let uri = client.get_service_uri().await.unwrap();
                println!("Service URI: {uri}");
            }
        },
        Command::ServiceHandler(ServiceHandlerArgs { command, address }) => match command {
            ServiceHandlerCommand::GetManager => {
                let client = ctx
                    .wavs_service_handler_query_client(address)
                    .await
                    .unwrap();
                let manager = client.get_manager_address().await.unwrap();
                println!("Service Manager: {manager}");
            }
        },
    }
}
