use std::path::{Path, PathBuf};
use tokio::sync::OnceCell;
use utils::path::repo_root;

use crate::client::pool::TestPool;

static MOCK_SERVICE_HANDLER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static MOCK_SERVICE_MANAGER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static ECDSA_SERVICE_HANDLER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static ECDSA_SERVICE_MANAGER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static BLS_SERVICE_HANDLER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static BLS_SERVICE_MANAGER_CODE_ID: OnceCell<u64> = OnceCell::const_new();
static SIMPLE_TRIGGER_CODE_ID: OnceCell<u64> = OnceCell::const_new();

pub struct CodeId {}

impl CodeId {
    pub async fn new_mock_service_handler() -> u64 {
        *MOCK_SERVICE_HANDLER_CODE_ID
            .get_or_init(upload_mock_service_handler)
            .await
    }
    pub async fn new_mock_service_manager() -> u64 {
        *MOCK_SERVICE_MANAGER_CODE_ID
            .get_or_init(upload_mock_service_manager)
            .await
    }

    pub async fn new_bls_service_handler() -> u64 {
        *BLS_SERVICE_HANDLER_CODE_ID
            .get_or_init(upload_bls_service_handler)
            .await
    }

    pub async fn new_bls_service_manager() -> u64 {
        *BLS_SERVICE_MANAGER_CODE_ID
            .get_or_init(upload_bls_service_manager)
            .await
    }

    pub async fn new_ecdsa_service_handler() -> u64 {
        *ECDSA_SERVICE_HANDLER_CODE_ID
            .get_or_init(upload_ecdsa_service_handler)
            .await
    }
    pub async fn new_ecdsa_service_manager() -> u64 {
        *ECDSA_SERVICE_MANAGER_CODE_ID
            .get_or_init(upload_ecdsa_service_manager)
            .await
    }
    pub async fn new_simple_trigger() -> u64 {
        *SIMPLE_TRIGGER_CODE_ID
            .get_or_init(upload_simple_trigger)
            .await
    }
}

async fn upload_simple_trigger() -> u64 {
    let wasm_path = repo_root()
        .unwrap()
        .join("packages")
        .join("contracts")
        .join("trigger")
        .join("artifacts")
        .join("simple_.wasm");

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

async fn upload(wasm_path: impl AsRef<Path>) -> u64 {
    let wasm_path = wasm_path.as_ref();

    let wasm_bytes = tokio::fs::read(&wasm_path)
        .await
        .unwrap_or_else(|_| panic!("Failed to read {}", wasm_path.display()));

    let pool = TestPool::get().await;
    let client = pool.pool.get().await.unwrap();

    let code_id = client
        .contract_upload_file(wasm_bytes, None)
        .await
        .unwrap()
        .0;

    tracing::info!("mock trigger code ID: {}", code_id);

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
