use cosmwasm_std::Addr;
use off_chain_tests::client::{mirror::MirrorTestClient, ContractTestClient};
use shared_tests::{mirror_stake_registry, tracing_init::tracing_tests_init};

#[tokio::test]
async fn test_mirror_operator_management() {
    tracing_tests_init();

    let client = ContractTestClient::new(Addr::unchecked("admin"));
    let mirror_client = MirrorTestClient::new(client);

    mirror_stake_registry::run_mirror_operator_management_test(
        &mirror_client.stake_registry_executor,
        &mirror_client.stake_registry_querier,
    )
    .await;
}

#[tokio::test]
async fn test_mirror_batch_operator_management() {
    tracing_tests_init();

    let client = ContractTestClient::new(Addr::unchecked("admin"));
    let mirror_client = MirrorTestClient::new(client);

    mirror_stake_registry::run_mirror_batch_operator_management_test(
        &mirror_client.stake_registry_executor,
        &mirror_client.stake_registry_querier,
    )
    .await;
}

#[tokio::test]
async fn test_mirror_abi_signature_validation() {
    tracing_tests_init();

    let client = ContractTestClient::new(Addr::unchecked("admin"));
    let mirror_client = MirrorTestClient::new(client);

    mirror_stake_registry::run_mirror_abi_signature_validation_test(
        &mirror_client.stake_registry_executor,
        &mirror_client.stake_registry_querier,
    )
    .await;
}

#[tokio::test]
async fn test_mirror_abi_binary_compatibility() {
    tracing_tests_init();

    mirror_stake_registry::run_mirror_abi_binary_compatibility_test().await;
}
