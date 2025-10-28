use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::command::{
    OutputFormat, RegistryContractKind, ServiceHandlerContractKind, ServiceManagerContractKind,
};

pub struct Output {
    pub path: Option<PathBuf>,
    pub format: OutputFormat,
}

impl Output {
    pub fn new(path: Option<PathBuf>, format: OutputFormat) -> Self {
        Self { path, format }
    }

    pub async fn write(&self, data: OutputData) -> Result<()> {
        if let Some(path) = &self.path {
            match self.format {
                OutputFormat::Json => {
                    let json_data = serde_json::to_string_pretty(&data)?;
                    tokio::fs::write(path, json_data).await?;
                }
            }
            tracing::info!("Output written to {}", path.display());
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged, rename_all = "snake_case")]
pub enum OutputData {
    ServiceManagerUpload {
        contract_kind: ServiceManagerContractKind,
        code_id: u64,
        tx_hash: String,
    },
    ServiceManagerInstantiate {
        contract_kind: ServiceManagerContractKind,
        address: String,
        tx_hash: String,
    },
    ServiceHandlerUpload {
        contract_kind: ServiceHandlerContractKind,
        code_id: u64,
        tx_hash: String,
    },
    ServiceHandlerInstantiate {
        contract_kind: ServiceHandlerContractKind,
        address: String,
        tx_hash: String,
    },
    RegistryUpload {
        contract_kind: RegistryContractKind,
        code_id: u64,
        tx_hash: String,
    },
    RegistryInstantiate {
        contract_kind: RegistryContractKind,
        registry_address: String,
        service_manager_address: String,
        tx_hash: String,
    },
}
