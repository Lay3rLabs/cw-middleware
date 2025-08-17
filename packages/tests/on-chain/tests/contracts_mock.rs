use on_chain_tests::client::contract::{
    mock::MockTestClient, simple_trigger::SimpleTriggerTestClient, ContractTestClient,
};
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn mock_sanity() {
    tracing_tests_init();

    let client = ContractTestClient::new().await;
    let simple_trigger = SimpleTriggerTestClient::new(client.clone()).await;
    let contract = MockTestClient::new(client.clone()).await;

    contracts_sanity::run_sanity_tests(&contract.wrap_test(&simple_trigger)).await;
}
