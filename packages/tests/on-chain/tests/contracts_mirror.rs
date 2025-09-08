use on_chain_tests::client::contract::{
    mirror::MirrorTestClient, trigger::SimpleTriggerTestClient, ContractTestClient,
};
use shared_tests::{contracts_sanity, mirror_stake_registry, tracing_init::tracing_tests_init};

#[tokio::test]
async fn mirror_sanity() {
    tracing_tests_init();

    let client = ContractTestClient::new().await;
    let trigger_simple = SimpleTriggerTestClient::new(client.clone()).await;
    let contract = MirrorTestClient::new(client.clone()).await;

    contracts_sanity::run_sanity_tests(&contract.wrap_test(&trigger_simple)).await;
}

#[tokio::test]
async fn test_mirror_stake_registry_sanity() {
    tracing_tests_init();

    let client = ContractTestClient::new().await;
    let contract = MirrorTestClient::new(client).await;

    mirror_stake_registry::run_mirror_sanity_tests(
        &contract.stake_registry_executor,
        &contract.stake_registry_querier,
    )
    .await;
}

#[tokio::test]
async fn test_mirror_negative_scenarios() {
    tracing_tests_init();

    let client = ContractTestClient::new().await;
    let contract = MirrorTestClient::new(client).await;

    mirror_stake_registry::run_mirror_negative_test_scenarios(
        &contract.stake_registry_executor,
        &contract.stake_registry_querier,
    )
    .await;
}
