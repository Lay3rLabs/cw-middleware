use off_chain_tests::client::{
    bls::BlsTestClient, trigger::SimpleTriggerTestClient, ContractTestClient,
};
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn bls_sanity() {
    tracing_tests_init();

    let client = ContractTestClient::new("admin");
    let bls_client = BlsTestClient::new(client.clone());
    let trigger_client = SimpleTriggerTestClient::new(client);

    contracts_sanity::run_sanity_tests(&bls_client.wrap_test(&trigger_client)).await;
}
