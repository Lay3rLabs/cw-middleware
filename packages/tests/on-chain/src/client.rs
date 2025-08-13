use std::{path::PathBuf, sync::LazyLock};

use cosmwasm_std::Empty;
use deadpool::managed::Pool;
use tokio::sync::OnceCell;

use layer_climb::{
    pool::{SigningClientPool, SigningClientPoolManager},
    prelude::*,
};
use utils::{
    client::on_chain::WavsSigningPoolClient,
    config::ChainConfigs,
    faucet,
    path::{repo_root, repo_wavs_home},
};

// The outside world only needs to see this
// everything else is local to deal with thread safety, pools, etc.
pub async fn new_mock_client() -> WavsSigningPoolClient {
    let (querier, pool) = deployer_pool().await;
    let manager_addr = deploy_service_manager().await;
    let handler_addr = deploy_service_handler(manager_addr.clone()).await;

    WavsSigningPoolClient::new(querier, pool, &handler_addr, &manager_addr)
}

//////////////////////////////////////////////////////////////////////////
const MAINTAIN_MINIMUM_BALANCE_THRESHOLD: u128 = 100000;
const MAINTAIN_MINIMUM_BALANCE_TOPUP: u128 = 10000000;

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

async fn deploy_service_manager() -> Address {
    let deployer = MOCK_DEPLOYER.get_or_init(MockDeployer::new).await;
    let client = deployer.pool.get().await.unwrap();

    let (addr, _) = client
        .contract_instantiate(
            None,
            deployer.mock_service_manager_code_id,
            "Service Manager",
            &Empty {},
            vec![],
            None,
        )
        .await
        .unwrap();

    tracing::info!("Deployed service manager at {}", addr);

    addr
}

async fn deploy_service_handler(service_manager: Address) -> Address {
    let deployer = MOCK_DEPLOYER.get_or_init(MockDeployer::new).await;
    let client = deployer.pool.get().await.unwrap();

    let (addr, _) = client
        .contract_instantiate(
            None,
            deployer.mock_service_handler_code_id,
            "Service Handler",
            &mock_api::service_handler::InstantiateMsg {
                service_manager: service_manager.to_string(),
            },
            vec![],
            None,
        )
        .await
        .unwrap();

    tracing::info!("Deployed service handler at {}", addr);

    addr
}

async fn deployer_pool() -> (QueryClient, SigningClientPool) {
    let deployer = MOCK_DEPLOYER.get_or_init(MockDeployer::new).await;
    (deployer.querier.clone(), deployer.pool.clone())
}

impl MockDeployer {
    pub async fn new() -> Self {
        let mnemonic = std::env::var("CLI_MNEMONIC").expect("CLI_MNEMONIC must be set");

        let chain_configs = ChainConfigs::load_from_wavs(repo_wavs_home())
            .await
            .expect("Failed to load chain configurations");

        let (_, chain_config) = chain_configs
            .cosmos
            .into_iter()
            .next()
            .expect("No chain configs found");
        let chain_config: ChainConfig = chain_config.into();
        let querier = QueryClient::new(chain_config.clone(), None).await.unwrap();

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

        let signer = KeySigner::new_mnemonic_str(&mnemonic, None)
            .expect("Failed to create KeySigner from mnemonic");

        let addr = chain_config
            .address_from_pub_key(&signer.public_key().await.unwrap())
            .unwrap();

        let balance = querier
            .balance(addr.clone(), None)
            .await
            .unwrap()
            .unwrap_or_default();

        if balance < 10000000000 {
            tracing::info!("{} has balance of {}, tapping faucet...", addr, balance);
            faucet::tap(&addr, &chain_config.gas_denom).await.unwrap();
            let new_balance = querier
                .balance(addr, None)
                .await
                .unwrap()
                .unwrap_or_default();
            if new_balance == balance {
                panic!("Failed to tap faucet, balance did not change");
            }
            tracing::info!("new balance is {:?}", new_balance);
        } else {
            tracing::info!("{} has balance of {}, no need to tap faucet", addr, balance);
        }

        let pool =
            SigningClientPoolManager::new_mnemonic(mnemonic, chain_config.clone(), None, None)
                .with_minimum_balance(
                    MAINTAIN_MINIMUM_BALANCE_THRESHOLD,
                    MAINTAIN_MINIMUM_BALANCE_TOPUP,
                    None,
                    None,
                )
                .await
                .unwrap();

        let pool = SigningClientPool::new(Pool::builder(pool).max_size(8).build().unwrap());

        let client = pool.get().await.unwrap();

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
            mock_service_handler_code_id,
            mock_service_manager_code_id,
            pool,
            querier,
        }
    }
}

#[derive(Clone)]
struct MockDeployer {
    mock_service_handler_code_id: u64,
    mock_service_manager_code_id: u64,
    pool: SigningClientPool,
    querier: QueryClient,
}

static MOCK_DEPLOYER: OnceCell<MockDeployer> = OnceCell::const_new();
