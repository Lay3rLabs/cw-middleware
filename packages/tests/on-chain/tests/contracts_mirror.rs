use on_chain_tests::client::contract::{
    mirror::MirrorTestClient, trigger::SimpleTriggerTestClient, ContractTestClient,
};
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn mirror_sanity() {
    tracing_tests_init();

    let client = ContractTestClient::new().await;
    let trigger_simple = SimpleTriggerTestClient::new(client.clone()).await;
    let contract = MirrorTestClient::new(client.clone()).await;

    contracts_sanity::run_sanity_tests(&contract.wrap_test(&trigger_simple)).await;
}
