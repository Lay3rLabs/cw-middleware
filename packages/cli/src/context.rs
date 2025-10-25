use anyhow::{Context, Result};
use clap::Parser;
use cw_wavs_sdk::{
    service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier},
    service_manager::{ServiceManagerExecutor, ServiceManagerQuerier},
};
use layer_climb::prelude::*;
use rand::prelude::*;
use utils::config::load_chain_configs_from_wavs;
use wavs_types::ChainConfigs;

use crate::command::CliArgs;

pub struct CliContext {
    pub args: CliArgs,
    pub rng: ThreadRng,
    pub chain_configs: ChainConfigs,
}

impl CliContext {
    pub async fn new() -> Self {
        if dotenvy::dotenv().is_err() {
            tracing::debug!("Failed to load .env file");
        }
        let args = CliArgs::parse();

        let chain_configs = load_chain_configs_from_wavs(args.wavs_home.as_ref())
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
            .get_chain(&self.args.chain)
            .clone()
            .context(format!("Chain config not found for {}", self.args.chain))?
            .to_cosmos_config()?;

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

    pub async fn query_client(&self) -> Result<QueryClient> {
        QueryClient::new(self.chain_config()?, None).await
    }

    pub async fn signing_client(&self) -> Result<SigningClient> {
        let query_client = self.query_client().await?;

        let signer = KeySigner::new_mnemonic_str(&self.client_mnemonic()?, None)?;
        let address = self
            .chain_config()?
            .address_from_pub_key(&signer.public_key().await?)?;

        let balance = query_client
            .balance(address.clone(), None)
            .await?
            .unwrap_or_default();
        if balance == 0 {
            tracing::warn!("Balance is ZERO, maybe tap the faucet!");
        }
        let signing_client = SigningClient::new(self.chain_config()?, signer, None).await?;

        Ok(signing_client)
    }

    pub async fn climb_command_any_client(&self) -> Result<AnyClient> {
        if self.client_mnemonic().is_ok() {
            tracing::info!("Using SigningClient");
            Ok(AnyClient::Signing(self.signing_client().await?))
        } else {
            tracing::info!("Using QueryClient");
            Ok(AnyClient::Query(self.query_client().await?))
        }
    }

    pub async fn wallet_addr(&self) -> Result<Address> {
        let mnemonic = self.client_mnemonic()?;
        let signer = KeySigner::new_mnemonic_str(&mnemonic, None)?;
        let address = self
            .chain_config()?
            .address_from_pub_key(&signer.public_key().await?)?;
        Ok(address)
    }

    pub async fn wavs_service_handler_querier(&self, addr: &str) -> Result<ServiceHandlerQuerier> {
        Ok(ServiceHandlerQuerier::new(
            self.query_client().await?.into(),
            self.parse_address(addr)?.try_into()?,
        ))
    }

    pub async fn wavs_service_manager_querier(&self, addr: &str) -> Result<ServiceManagerQuerier> {
        Ok(ServiceManagerQuerier::new(
            self.query_client().await?.into(),
            self.parse_address(addr)?.try_into()?,
        ))
    }

    #[allow(dead_code)]
    pub async fn wavs_service_handler_executor(
        &self,
        addr: &str,
    ) -> Result<ServiceHandlerExecutor> {
        Ok(ServiceHandlerExecutor::new(
            self.signing_client().await?.into(),
            self.parse_address(addr)?.try_into()?,
        ))
    }

    #[allow(dead_code)]
    pub async fn wavs_service_manager_executor(
        &self,
        addr: &str,
    ) -> Result<ServiceManagerExecutor> {
        Ok(ServiceManagerExecutor::new(
            self.signing_client().await?.into(),
            self.parse_address(addr)?.try_into()?,
        ))
    }

    pub fn parse_address(&self, addr: &str) -> Result<Address> {
        self.chain_config()?.parse_address(addr)
    }
}
