pub mod contract;
pub mod wallet;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use layer_climb_cli::command::{ContractCommand, WalletCommand};
use wavs_types::ChainKey;

#[derive(Clone, Debug, Parser)]
pub struct CliArgs {
    /// If not set, will be backend/wavs-home in the repo root
    #[clap(long, env = "WAVS_HOME")]
    pub wavs_home: Option<PathBuf>,

    #[clap(long, default_value = "local", env = "CHAIN_KEY")]
    pub chain: ChainKey,
}

#[derive(Clone, Parser)]
#[command(version, about, long_about = None)]
pub enum Command {
    /// Generate mnemonics for the .env file
    GenerateEnv {
        operators: usize,
        #[clap(flatten)]
        args: CliArgs,
    },
    /// Wallet subcommands
    Wallet {
        #[command(subcommand)]
        command: WalletCommand,
        // we need to flatten at this level, can't edit WalletCommand
        // so for these climb commands we have to pass CliArgs first
        #[clap(flatten)]
        args: CliArgs,
    },
    /// Contract subcommands
    Contract {
        #[command(subcommand)]
        command: ContractCommand,
        // we need to flatten at this level, can't edit WalletCommand
        // so for these climb commands we have to pass CliArgs first
        #[clap(flatten)]
        args: CliArgs,
    },
    /// Service Manager subcommands
    ServiceManager {
        #[command(subcommand)]
        command: ServiceManagerCommand,
    },
    /// Service Handler subcommands
    ServiceHandler {
        #[command(subcommand)]
        command: ServiceHandlerCommand,
    },
    /// Registry subcommands
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
    /// Tap the faucet
    FaucetTap {
        /// If none, will be CLI wallet
        addr: Option<String>,
        #[arg(
            long,
            env = "FAUCET_URL",
            default_value = "http://localhost:8001/credit"
        )]
        url: String,
        #[clap(flatten)]
        args: CliArgs,
    },
}

impl Command {
    pub fn args(&self) -> &CliArgs {
        match self {
            Command::GenerateEnv { args, .. } => args,
            Command::Wallet { args, .. } => args,
            Command::Contract { args, .. } => args,
            Command::ServiceManager { command } => match command {
                ServiceManagerCommand::Upload { args, .. } => args,
                ServiceManagerCommand::InstantiateMock { args, .. } => args,
                ServiceManagerCommand::InstantiateEcdsa { args, .. } => args,
                ServiceManagerCommand::InstantiateBls { args, .. } => args,
                ServiceManagerCommand::InstantiateMirror { args, .. } => args,
                ServiceManagerCommand::SetServiceUri { args, .. } => args,
                ServiceManagerCommand::GetServiceUri { args, .. } => args,
            },
            Command::ServiceHandler { command } => match command {
                ServiceHandlerCommand::Upload { args, .. } => args,
                ServiceHandlerCommand::InstantiateMock { args, .. } => args,
                ServiceHandlerCommand::InstantiateEcdsa { args, .. } => args,
                ServiceHandlerCommand::InstantiateBls { args, .. } => args,
                ServiceHandlerCommand::InstantiateMirror { args, .. } => args,
                ServiceHandlerCommand::GetManager { args, .. } => args,
            },
            Command::Registry { command } => match command {
                RegistryCommand::Upload { args, .. } => args,
                RegistryCommand::InstantiateMirrorStake { args, .. } => args,
                RegistryCommand::GetServiceManager { args, .. } => args,
            },
            Command::FaucetTap { args, .. } => args,
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceManagerCommand {
    /// Upload a precompiled service manager contract
    Upload {
        // this is set to make the docker experience easier
        // and we keep the ease-of-use in taskfiles by overriding it there
        #[arg(long, env = "WASM_DIR", default_value = "/wasm/built-in")]
        wasm_directory: String,
        #[arg(long)]
        contract_kind: ServiceManagerContractKind,
        #[clap(flatten)]
        args: CliArgs,
    },

    /// Instantiate an instance of the mock service manager
    InstantiateMock {
        #[arg(long)]
        code_id: u64,
        #[clap(flatten)]
        args: CliArgs,
    },

    /// Instantiate an instance of the ecdsa aservice manager
    InstantiateEcdsa {
        #[arg(long)]
        code_id: u64,
        #[clap(flatten)]
        args: CliArgs,
    },

    /// Instantiate an instance of the bls service manager
    InstantiateBls {
        #[arg(long)]
        code_id: u64,
        #[clap(flatten)]
        args: CliArgs,
    },

    /// Instantiate an instance of the mirror service manager
    /// Requires admin address (typically you instead instantiate a Registry)
    InstantiateMirror {
        #[arg(long)]
        code_id: u64,
        #[arg(long)]
        owner: String,
        #[clap(flatten)]
        args: CliArgs,
    },

    /// Sets the service URI on the service manager contract
    SetServiceUri {
        #[arg(long)]
        uri: String,
        /// Service Manager address
        #[arg(long)]
        address: String,
        #[clap(flatten)]
        args: CliArgs,
    },

    /// Gets the service URI from the service manager contract
    GetServiceUri {
        /// Service Manager address
        #[arg(long)]
        address: String,
        #[clap(flatten)]
        args: CliArgs,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceHandlerCommand {
    /// Upload a precompiled service handler contract
    Upload {
        // this is set to make the docker experience easier
        // and we keep the ease-of-use in taskfiles by overriding it there
        #[arg(long, env = "WASM_DIR", default_value = "/wasm/built-in")]
        wasm_directory: String,
        #[arg(long)]
        contract_kind: ServiceHandlerContractKind,
        #[clap(flatten)]
        args: CliArgs,
    },
    /// Instantiate an instance of the mock service handler
    InstantiateMock {
        #[arg(long)]
        code_id: u64,
        #[arg(long)]
        service_manager: String,
        #[clap(flatten)]
        args: CliArgs,
    },
    /// Instantiate an instance of the ecsda service handler
    InstantiateEcdsa {
        #[arg(long)]
        code_id: u64,
        #[arg(long)]
        service_manager: String,
        #[clap(flatten)]
        args: CliArgs,
    },
    /// Instantiate an instance of the bls service handler
    InstantiateBls {
        #[arg(long)]
        code_id: u64,
        #[arg(long)]
        service_manager: String,
        #[clap(flatten)]
        args: CliArgs,
    },
    /// Instantiate an instance of the mirror service handler
    InstantiateMirror {
        #[arg(long)]
        code_id: u64,
        #[arg(long)]
        service_manager: String,
        #[clap(flatten)]
        args: CliArgs,
    },

    /// Gets the service manager address for this service handler
    GetManager {
        /// Service Handler address
        #[arg(long)]
        address: String,
        #[clap(flatten)]
        args: CliArgs,
    },
}

#[derive(Clone, Args)]
pub struct RegistryArgs {}

#[derive(Debug, Clone, Subcommand)]
pub enum RegistryCommand {
    /// Upload a precompiled registry contract
    Upload {
        // this is set to make the docker experience easier
        // and we keep the ease-of-use in taskfiles by overriding it there
        #[arg(long, env = "WASM_DIR", default_value = "/wasm/built-in")]
        wasm_directory: String,
        #[arg(long)]
        contract_kind: RegistryContractKind,
        #[clap(flatten)]
        args: CliArgs,
    },
    /// Instantiate an instance of the mirror stake registry
    InstantiateMirrorStake {
        #[arg(long)]
        code_id: u64,
        #[arg(long)]
        service_manager_code_id: u64,
        #[arg(long)]
        threshold_weight: u128,
        /// Configuration pairs for the strategies in format 'strategy=multiplier'
        /// Example: --strategy test_strategy=100 some_other_strategy=200
        #[arg(long, required = true, num_args = 1..)]
        strategy: Vec<String>,
        #[clap(flatten)]
        args: CliArgs,
    },
    /// Gets the service manager address for this registry
    GetServiceManager {
        /// Registry address
        #[arg(long)]
        address: String,
        #[arg(long)]
        contract_kind: RegistryContractKind,
        #[clap(flatten)]
        args: CliArgs,
    },
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum ServiceHandlerContractKind {
    Mock,
    Ecdsa,
    Bls,
    Mirror,
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum ServiceManagerContractKind {
    Mock,
    Ecdsa,
    Bls,
    Mirror,
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum RegistryContractKind {
    MirrorStake,
}

impl ServiceHandlerContractKind {
    pub fn wasm_path(&self, wasm_directory: &str) -> String {
        format!("{wasm_directory}/cw_wavs_{self}_service_handler.wasm")
    }
}

impl std::fmt::Display for ServiceHandlerContractKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceHandlerContractKind::Mock => write!(f, "mock"),
            ServiceHandlerContractKind::Ecdsa => write!(f, "ecdsa"),
            ServiceHandlerContractKind::Bls => write!(f, "bls"),
            ServiceHandlerContractKind::Mirror => write!(f, "mirror"),
        }
    }
}

impl ServiceManagerContractKind {
    pub fn wasm_path(&self, wasm_directory: &str) -> String {
        format!("{wasm_directory}/cw_wavs_{self}_service_manager.wasm")
    }
}

impl std::fmt::Display for ServiceManagerContractKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceManagerContractKind::Mock => write!(f, "mock"),
            ServiceManagerContractKind::Ecdsa => write!(f, "ecdsa"),
            ServiceManagerContractKind::Bls => write!(f, "bls"),
            ServiceManagerContractKind::Mirror => write!(f, "mirror"),
        }
    }
}

impl std::fmt::Display for RegistryContractKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryContractKind::MirrorStake => write!(f, "mirror_stake"),
        }
    }
}

impl RegistryContractKind {
    pub fn wasm_path(&self, wasm_directory: &str) -> String {
        format!("{wasm_directory}/cw_wavs_{self}_registry.wasm")
    }
}
