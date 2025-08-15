use std::path::Path;
use tokio::sync::OnceCell;
use interface::*;
use utils::path::repo_root;

use crate::client::pool::TestPool;

static MOCK_CODE_IDS: OnceCell<CodeIds> = OnceCell::const_new();
static ECDSA_CODE_IDS: OnceCell<CodeIds> = OnceCell::const_new();
static BLS_CODE_IDS: OnceCell<CodeIds> = OnceCell::const_new();
static MOCK_TRIGGER_CODE_ID: OnceCell<u64> = OnceCell::const_new();

#[derive(Clone)]
pub struct MockContractClient {
    app: TestApp,
}

impl MockContractClient {
    pub async fn new() -> Self {
        let code_ids = CodeIds::new_mock().await;
        let trigger_code_id = CodeIds::new_mock_trigger().await;

        let TestPool { querier, pool } = TestPool::get().await;
        let client = pool.get().await.unwrap();

        let (manager_addr, _) = client
            .contract_instantiate(
                None,
                code_ids.service_manager,
                "Mock Service Manager",
                &mock_api::service_manager::InstantiateMsg {},
                vec![],
                None,
            )
            .await
            .unwrap();

        let (handler_addr, _) = client
            .contract_instantiate(
                None,
                code_ids.service_handler,
                "Mock Service Handler",
                &mock_api::service_handler::InstantiateMsg {
                    service_manager: manager_addr.to_string(),
                },
                vec![],
                None,
            )
            .await
            .unwrap();

        let (trigger_addr, _) = client
            .contract_instantiate(
                None,
                trigger_code_id,
                "Mock Trigger",
                &trigger_api::simple::InstantiateMsg {},
                vec![],
                None,
            )
            .await
            .unwrap();

        let app =
            TestApp::new(querier, pool, &handler_addr, &manager_addr, &trigger_addr);

        Self { app}
    }
}

#[derive(Clone)]
pub struct EcdsaContractClient {
    app: TestApp,
}

impl EcdsaContractClient {
    pub async fn new() -> Self {
        let code_ids = CodeIds::new_ecdsa().await;
        let TestPool { querier, pool } = TestPool::get().await;
        let client = pool.get().await.unwrap();

        let (manager_addr, _) = client
            .contract_instantiate(
                None,
                code_ids.service_manager,
                "Ecdsa Service Manager",
                &ecdsa_api::service_manager::InstantiateMsg {},
                vec![],
                None,
            )
            .await
            .unwrap();

        let (handler_addr, _) = client
            .contract_instantiate(
                None,
                code_ids.service_handler,
                "Ecdsa Service Handler",
                &ecdsa_api::service_handler::InstantiateMsg {
                    service_manager: manager_addr.to_string(),
                },
                vec![],
                None,
            )
            .await
            .unwrap();

        let (trigger_addr, _) = client
            .contract_instantiate(
                None,
                CodeIds::new_mock_trigger().await,
                "Mock Trigger",
                &trigger_api::simple::InstantiateMsg {},
                vec![],
                None,
            )
            .await
            .unwrap();

        let app =
            TestApp::new(querier, pool, &handler_addr, &manager_addr, &trigger_addr);

        Self { app }
    }
}

#[derive(Clone)]
pub struct BlsContractClient {
    app: TestApp
}

impl BlsContractClient {
    pub async fn new() -> Self {
        let code_ids = CodeIds::new_bls().await;
        let TestPool { querier, pool } = TestPool::get().await;
        let client = pool.get().await.unwrap();

        let (manager_addr, _) = client
            .contract_instantiate(
                None,
                code_ids.service_manager,
                "BLS Service Manager",
                &bls_api::service_manager::InstantiateMsg {},
                vec![],
                None,
            )
            .await
            .unwrap();

        let (handler_addr, _) = client
            .contract_instantiate(
                None,
                code_ids.service_handler,
                "BLS Service Handler",
                &bls_api::service_handler::InstantiateMsg {
                    service_manager: manager_addr.to_string(),
                },
                vec![],
                None,
            )
            .await
            .unwrap();

        let (trigger_addr, _) = client
            .contract_instantiate(
                None,
                CodeIds::new_mock_trigger().await,
                "Mock Trigger",
                &trigger_api::simple::InstantiateMsg {},
                vec![],
                None,
            )
            .await
            .unwrap();

        let app =
            TestApp::new(querier, pool, &handler_addr, &manager_addr, &trigger_addr);

        Self { app }
    }
}

#[derive(Clone)]
struct CodeIds {
    service_handler: u64,
    service_manager: u64,
}

impl CodeIds {
    pub async fn new_mock() -> Self {
        MOCK_CODE_IDS
            .get_or_init(Self::instantiate_mock)
            .await
            .clone()
    }

    pub async fn new_ecdsa() -> Self {
        ECDSA_CODE_IDS
            .get_or_init(Self::instantiate_ecdsa)
            .await
            .clone()
    }

    pub async fn new_bls() -> Self {
        BLS_CODE_IDS
            .get_or_init(Self::instantiate_bls)
            .await
            .clone()
    }

    pub async fn new_mock_trigger() -> u64 {
        *MOCK_TRIGGER_CODE_ID
            .get_or_init(Self::instantiate_mock_trigger)
            .await
    }

    async fn instantiate_mock() -> Self {
        let artifacts_path = repo_root()
            .unwrap()
            .join("packages")
            .join("contracts")
            .join("mock")
            .join("artifacts");

        let service_handler_path = artifacts_path.join("mock_service_handler.wasm");
        let service_manager_path = artifacts_path.join("mock_service_manager.wasm");
        Self::instantiate(service_handler_path, service_manager_path).await
    }

    async fn instantiate_ecdsa() -> Self {
        let artifacts_path = repo_root()
            .unwrap()
            .join("packages")
            .join("contracts")
            .join("ecdsa")
            .join("artifacts");

        let service_handler_path = artifacts_path.join("ecdsa_service_handler.wasm");
        let service_manager_path = artifacts_path.join("ecdsa_service_manager.wasm");
        Self::instantiate(service_handler_path, service_manager_path).await
    }

    async fn instantiate_bls() -> Self {
        let artifacts_path = repo_root()
            .unwrap()
            .join("packages")
            .join("contracts")
            .join("bls")
            .join("artifacts");

        let service_handler_path = artifacts_path.join("bls_service_handler.wasm");
        let service_manager_path = artifacts_path.join("bls_service_manager.wasm");
        Self::instantiate(service_handler_path, service_manager_path).await
    }

    async fn instantiate(
        path_to_service_handler: impl AsRef<Path>,
        path_to_service_manager: impl AsRef<Path>,
    ) -> Self {
        let handler_wasm_bytes = tokio::fs::read(path_to_service_handler.as_ref())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to read {}",
                    path_to_service_handler.as_ref().display()
                )
            });
        let manager_wasm_bytes = tokio::fs::read(path_to_service_manager.as_ref())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to read {}",
                    path_to_service_manager.as_ref().display()
                )
            });

        let pool = TestPool::get().await;
        let client = pool.pool.get().await.unwrap();

        let service_handler_code_id = client
            .contract_upload_file(handler_wasm_bytes, None)
            .await
            .unwrap()
            .0;

        tracing::info!("service handler code ID: {}", service_handler_code_id);

        let service_manager_code_id = client
            .contract_upload_file(manager_wasm_bytes, None)
            .await
            .unwrap()
            .0;

        tracing::info!("service manager code ID: {}", service_manager_code_id);

        Self {
            service_handler: service_handler_code_id,
            service_manager: service_manager_code_id,
        }
    }

    async fn instantiate_mock_trigger() -> u64 {
        let wasm_path = repo_root()
            .unwrap()
            .join("packages")
            .join("contracts")
            .join("mock")
            .join("artifacts")
            .join("mock_trigger.wasm")
            .to_path_buf();

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
}