pub mod contract;
pub mod wallet;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use layer_climb_cli::command::{ContractCommand, WalletCommand};
use wavs_types::ChainName;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// If not set, will be backend/wavs-home in the repo root
    #[clap(long)]
    pub wavs_home: Option<PathBuf>,

    #[clap(long, default_value = "local")]
    pub chain_name: ChainName,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Subcommand)]
pub enum Command {
    /// Generate mnemonics for the .env file
    GenerateEnv { operators: usize },
    /// Wallet subcommands
    Wallet(WalletArgs),
    /// Contract subcommands
    Contract(ContractArgs),
    /// Service Manager subcommands
    ServiceManager(ServiceManagerArgs),
    /// Service Handler subcommands
    ServiceHandler(ServiceHandlerArgs),
    /// Tap the faucet for
    FaucetTap {
        /// If none, will be CLI wallet
        addr: Option<String>,
    },
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
    #[command(subcommand)]
    pub command: ServiceManagerCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceManagerCommand {
    /// Deploy an instance of the service manager
    Deploy {
        #[arg(long)]
        code_id: u64,
        #[arg(long)]
        contract_kind: ContractKind,
    },

    /// Sets the service URI on the service manager contract
    SetServiceUri {
        #[arg(long)]
        uri: String,
        /// Service Manager address
        #[arg(long)]
        address: String,
    },

    /// Gets the service URI from the service manager contract
    GetServiceUri {
        /// Service Manager address
        #[arg(long)]
        address: String,
    },
}

#[derive(Clone, Args)]
pub struct ServiceHandlerArgs {
    #[command(subcommand)]
    pub command: ServiceHandlerCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceHandlerCommand {
    /// Deploy an instance of the service handler
    Deploy {
        #[arg(long)]
        code_id: u64,
        #[arg(long)]
        service_manager: String,
        #[arg(long)]
        contract_kind: ContractKind,
    },

    /// Gets the service manager address for this service manager
    GetManager {
        /// Service Handler address
        #[arg(long)]
        address: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum ContractKind {
    Mock,
    Ecdsa,
    Bls,
}
