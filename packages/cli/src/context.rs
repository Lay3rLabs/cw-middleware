use anyhow::{Context, Result};
use clap::Parser;
use layer_climb::prelude::*;
use layer_climb_cli::command::WalletCommand;
use rand::prelude::*;
use utils::{
    client::on_chain::{
        WavsServiceHandlerQueryClient, WavsServiceHandlerSigningClient,
        WavsServiceManagerQueryClient, WavsServiceManagerSigningClient,
    },
    config::ChainConfigs,
};

use crate::command::{CliArgs, Command, WalletArgs};

pub struct CliContext {
    pub args: CliArgs,
    pub rng: ThreadRng,
    pub chain_configs: ChainConfigs,
}

impl CliContext {
    pub async fn new() -> Self {
        if dotenvy::dotenv().is_err() {
            eprintln!("Failed to load .env file");
        }
        let args = CliArgs::parse();

        let chain_configs = ChainConfigs::load_from_wavs(args.wavs_home.as_ref())
            .await
            .expect("Failed to load chain configurations");

        Self {
            args,
            rng: rand::rng(),
            chain_configs,
        }
    }

    pub fn chain_config(&self) -> Result<ChainConfig> {
        let chain_config = self
            .chain_configs
            .cosmos
            .get(&self.args.chain_name)
            .cloned()
            .context(format!(
                "Chain config not found for {}",
                self.args.chain_name
            ))?;
        Ok(chain_config.into())
    }

    pub fn client_mnemonic(&self) -> Result<String> {
        std::env::var("CLI_MNEMONIC")
            .and_then(|m| {
                if m.is_empty() {
                    Err(std::env::VarError::NotPresent)
                } else {
                    Ok(m)
                }
            })
            .context("Mnemonic not found at CLI_MNEMONIC".to_string())
    }

    pub async fn any_client(&self) -> Result<AnyClient> {
        if matches!(
            &self.args.command,
            Command::Wallet(WalletArgs {
                command: WalletCommand::Create
            })
        ) {
            tracing::info!("Using QueryClient");
        } else {
            tracing::info!("Using SigningClient");
        }
        let is_signing = !matches!(&self.args.command, Command::Wallet(_));

        tracing::info!("IS SIGNING: {}", is_signing);

        match (is_signing, self.client_mnemonic()) {
            (true, Ok(mnemonic)) => {
                let signer = KeySigner::new_mnemonic_str(&mnemonic, None)?;
                Ok(AnyClient::Signing(
                    SigningClient::new(self.chain_config()?, signer, None).await?,
                ))
            }
            _ => Ok(AnyClient::Query(
                QueryClient::new(self.chain_config()?, None).await?,
            )),
        }
    }

    pub async fn wavs_service_handler_query_client(
        &self,
        addr: &str,
    ) -> Result<WavsServiceHandlerQueryClient> {
        Ok(WavsServiceHandlerQueryClient::new(
            self.any_client().await?.as_querier().clone(),
            &self.parse_address(addr)?,
        ))
    }

    pub async fn wavs_service_manager_query_client(
        &self,
        addr: &str,
    ) -> Result<WavsServiceManagerQueryClient> {
        Ok(WavsServiceManagerQueryClient::new(
            self.any_client().await?.as_querier().clone(),
            &self.parse_address(addr)?,
        ))
    }

    #[allow(dead_code)]
    pub async fn wavs_service_handler_signing_client(
        &self,
        addr: &str,
    ) -> Result<WavsServiceHandlerSigningClient> {
        Ok(WavsServiceHandlerSigningClient::new(
            self.any_client().await?.as_signing().clone(),
            &self.parse_address(addr)?,
        ))
    }

    pub async fn wavs_service_manager_signing_client(
        &self,
        addr: &str,
    ) -> Result<WavsServiceManagerSigningClient> {
        Ok(WavsServiceManagerSigningClient::new(
            self.any_client().await?.as_signing().clone(),
            &self.parse_address(addr)?,
        ))
    }

    pub fn parse_address(&self, addr: &str) -> Result<Address> {
        self.chain_config()?.parse_address(addr)
    }
}
