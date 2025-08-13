pub mod contract;
pub mod wallet;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use layer_climb_cli::command::{ContractCommand, WalletCommand};
use wavs_types::ChainName;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// If not set, will be backend/wavs-home in the repo root
    #[clap(long)]
    pub wavs_home: Option<PathBuf>,

    #[clap(long, default_value = "uni-7")]
    pub chain_name: ChainName,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Subcommand)]
pub enum Command {
    /// Generate mnemonics for the .env file
    GenerateEnv,
    /// Wallet subcommands
    Wallet(WalletArgs),
    /// Contract subcommands
    Contract(ContractArgs),
    /// Service Manager subcommands
    ServiceManager(ServiceManagerArgs),
    /// Service Handler subcommands
    ServiceHandler(ServiceHandlerArgs),
}

#[derive(Clone, Args)]
pub struct WalletArgs {
    #[command(subcommand)]
    pub command: WalletCommand,
}

#[derive(Clone, Args)]
pub struct ContractArgs {
    #[command(subcommand)]
    pub command: ContractCommand,
}

#[derive(Clone, Args)]
pub struct ServiceManagerArgs {
    #[clap(long)]
    pub address: String,
    #[command(subcommand)]
    pub command: ServiceManagerCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceManagerCommand {
    /// Sets the service URI on the service manager contract
    SetServiceUri {
        #[arg(long)]
        uri: String,
    },

    /// Gets the service URI from the service manager contract
    GetServiceUri,
}

#[derive(Clone, Args)]
pub struct ServiceHandlerArgs {
    #[clap(long)]
    pub address: String,
    #[command(subcommand)]
    pub command: ServiceHandlerCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceHandlerCommand {
    /// Gets the service manager address for this service manager
    GetManager,
}
