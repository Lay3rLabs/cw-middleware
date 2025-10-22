use on_chain_tests::client::contract::{
    bls::BlsTestClient, trigger::SimpleTriggerTestClient, ContractTestClient,
};
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn mock_sanity() {
    tracing_tests_init();

    let client = ContractTestClient::new().await;
    let cw_wavs_trigger_simple = SimpleTriggerTestClient::new(client.clone()).await;
    let contract = BlsTestClient::new(client.clone()).await;

    contracts_sanity::run_sanity_tests(&contract.wrap_test(&cw_wavs_trigger_simple)).await;
}
