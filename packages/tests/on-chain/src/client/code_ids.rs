use std::path::{Path, PathBuf};
use tokio::sync::OnceCell;
use tracing::{debug, info, instrument};
use utils::path::repo_root;

use crate::client::pool::TestPool;

static MOCK_SERVICE_HANDLER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static MOCK_SERVICE_MANAGER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static ECDSA_SERVICE_HANDLER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static ECDSA_SERVICE_MANAGER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static BLS_SERVICE_HANDLER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static BLS_SERVICE_MANAGER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static MIRROR_SERVICE_HANDLER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static MIRROR_SERVICE_MANAGER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static MIRROR_STAKE_REGISTRY_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static TRIGGER_SIMPLE_CODE_ID: OnceCell<u64> = OnceCell::const_new();

pub struct CodeId {}

impl CodeId {
    #[instrument]
    pub async fn new_mock_service_handler() -> u64 {
        *MOCK_SERVICE_HANDLER_CODE_ID
            .get_or_init(upload_mock_service_handler)
            .await
    }
    #[instrument]
    pub async fn new_mock_service_manager() -> u64 {
        *MOCK_SERVICE_MANAGER_CODE_ID
            .get_or_init(upload_mock_service_manager)
            .await
    }

    #[instrument]
    pub async fn new_bls_service_handler() -> u64 {
        *BLS_SERVICE_HANDLER_CODE_ID
            .get_or_init(upload_bls_service_handler)
            .await
    }

    #[instrument]
    pub async fn new_bls_service_manager() -> u64 {
        *BLS_SERVICE_MANAGER_CODE_ID
            .get_or_init(upload_bls_service_manager)
            .await
    }

    #[instrument]
    pub async fn new_ecdsa_service_handler() -> u64 {
        *ECDSA_SERVICE_HANDLER_CODE_ID
            .get_or_init(upload_ecdsa_service_handler)
            .await
    }
    #[instrument]
    pub async fn new_ecdsa_service_manager() -> u64 {
        *ECDSA_SERVICE_MANAGER_CODE_ID
            .get_or_init(upload_ecdsa_service_manager)
            .await
    }

    #[instrument]
    pub async fn new_mirror_service_handler() -> u64 {
        *MIRROR_SERVICE_HANDLER_CODE_ID
            .get_or_init(upload_mirror_service_handler)
            .await
    }

    #[instrument]
    pub async fn new_mirror_service_manager() -> u64 {
        *MIRROR_SERVICE_MANAGER_CODE_ID
            .get_or_init(upload_mirror_service_manager)
            .await
    }

    #[instrument]
    pub async fn new_mirror_stake_registry() -> u64 {
        *MIRROR_STAKE_REGISTRY_CODE_ID
            .get_or_init(upload_mirror_stake_registry)
            .await
    }

    #[instrument]
    pub async fn new_trigger_simple() -> u64 {
        *TRIGGER_SIMPLE_CODE_ID
            .get_or_init(upload_trigger_simple)
            .await
    }
}

async fn upload_trigger_simple() -> u64 {
    let wasm_path = repo_root()
        .unwrap()
        .join("packages")
        .join("contracts")
        .join("trigger")
        .join("artifacts")
        .join("trigger_simple.wasm");

    upload(wasm_path).await
}

async fn upload_mock_service_handler() -> u64 {
    upload(service_wasm_path("mock", "service_handler")).await
}

async fn upload_mock_service_manager() -> u64 {
    upload(service_wasm_path("mock", "service_manager")).await
}

async fn upload_ecdsa_service_handler() -> u64 {
    upload(service_wasm_path("ecdsa", "service_handler")).await
}

async fn upload_ecdsa_service_manager() -> u64 {
    upload(service_wasm_path("ecdsa", "service_manager")).await
}

async fn upload_bls_service_handler() -> u64 {
    upload(service_wasm_path("bls", "service_handler")).await
}

async fn upload_bls_service_manager() -> u64 {
    upload(service_wasm_path("bls", "service_manager")).await
}

async fn upload_mirror_service_handler() -> u64 {
    upload(service_wasm_path("mirror", "service_handler")).await
}

async fn upload_mirror_service_manager() -> u64 {
    upload(service_wasm_path("mirror", "service_manager")).await
}

async fn upload_mirror_stake_registry() -> u64 {
    let wasm_path = repo_root()
        .unwrap()
        .join("packages")
        .join("contracts")
        .join("mirror")
        .join("artifacts")
        .join("mirror_stake_registry.wasm");

    upload(wasm_path).await
}

#[instrument(skip(wasm_path), fields(path = %wasm_path.as_ref().display()))]
async fn upload(wasm_path: impl AsRef<Path>) -> u64 {
    let wasm_path = wasm_path.as_ref();

    info!("Reading WASM file");
    let wasm_bytes = tokio::fs::read(&wasm_path)
        .await
        .unwrap_or_else(|_| panic!("Failed to read {}", wasm_path.display()));

    debug!(size_bytes = wasm_bytes.len(), "WASM file loaded");

    let pool = TestPool::get().await;
    let client = pool.pool.get().await.unwrap();

    debug!("Uploading contract to chain");
    let code_id = client
        .contract_upload_file(wasm_bytes, None)
        .await
        .unwrap()
        .0;

    info!(code_id, "Contract uploaded successfully");

    code_id
}

fn service_wasm_path(contract_kind: &str, service_kind: &str) -> PathBuf {
    let artifacts_path = repo_root()
        .unwrap()
        .join("packages")
        .join("contracts")
        .join(contract_kind)
        .join("artifacts");

    artifacts_path.join(format!("{contract_kind}_{service_kind}.wasm"))
}
