use std::{path::PathBuf, sync::LazyLock};

use cosmwasm_std::Empty;
use tokio::sync::OnceCell;

use utils::{
    client::on_chain::WavsSigningPoolClient,
    path::repo_root
};

use crate::client::pool::TestPool;

static MOCK_CODE_IDS: OnceCell<MockCodeIds> = OnceCell::const_new();

static PATH_TO_MOCK_ARTIFACTS: LazyLock<PathBuf> = LazyLock::new(|| {
    repo_root()
        .unwrap()
        .join("packages")
        .join("contracts")
        .join("mock")
        .join("artifacts")
});

static PATH_TO_MOCK_SERVICE_HANDLER: LazyLock<PathBuf> =
    LazyLock::new(|| PATH_TO_MOCK_ARTIFACTS.join("mock_service_handler.wasm"));

static PATH_TO_MOCK_SERVICE_MANAGER: LazyLock<PathBuf> =
    LazyLock::new(|| PATH_TO_MOCK_ARTIFACTS.join("mock_service_manager.wasm"));

// The outside world only needs to see this
// everything else is local to deal with thread safety, pools, etc.
pub async fn new_mock_client() -> WavsSigningPoolClient {
    let code_ids = MockCodeIds::new().await;
    let TestPool{ querier, pool } = TestPool::get().await;
    let client = pool.get().await.unwrap();

    let (manager_addr, _) = client
        .contract_instantiate(
            None,
            code_ids.service_manager,
            "Service Manager",
            &Empty {},
            vec![],
            None,
        )
        .await
        .unwrap();

    let (handler_addr, _) = client
        .contract_instantiate(
            None,
            code_ids.service_handler,
            "Service Handler",
            &mock_api::service_handler::InstantiateMsg {
                service_manager: manager_addr.to_string(),
            },
            vec![],
            None,
        )
        .await
        .unwrap();

    WavsSigningPoolClient::new(querier, pool, &handler_addr, &manager_addr)
}


#[derive(Clone)]
struct MockCodeIds {
    service_handler: u64,
    service_manager: u64,
}

impl MockCodeIds {
    pub async fn new() -> Self {
        MOCK_CODE_IDS.get_or_init(Self::instantiate).await.clone()
    }

    async fn instantiate() -> Self {
        let mock_handler_wasm_bytes = tokio::fs::read(&*PATH_TO_MOCK_SERVICE_HANDLER)
            .await
            .unwrap_or_else(|_| {
                panic!("Failed to read {}", PATH_TO_MOCK_SERVICE_HANDLER.display())
            });
        let mock_manager_wasm_bytes = tokio::fs::read(&*PATH_TO_MOCK_SERVICE_MANAGER)
            .await
            .unwrap_or_else(|_| {
                panic!("Failed to read {}", PATH_TO_MOCK_SERVICE_MANAGER.display())
            });

        let pool = TestPool::get().await;
        let client = pool.pool.get().await.unwrap();


        let mock_service_handler_code_id = client
            .contract_upload_file(mock_handler_wasm_bytes, None)
            .await
            .unwrap()
            .0;
        tracing::info!(
            "Mock service handler code ID: {}",
            mock_service_handler_code_id
        );
        let mock_service_manager_code_id = client
            .contract_upload_file(mock_manager_wasm_bytes, None)
            .await
            .unwrap()
            .0;
        tracing::info!(
            "Mock service manager code ID: {}",
            mock_service_manager_code_id
        );


        Self {
            service_handler: mock_service_handler_code_id,
            service_manager: mock_service_manager_code_id
        }
    }
}

