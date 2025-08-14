use off_chain_tests::client::TestEcdsaClient;
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn ecdsa_sanity() {
    tracing_tests_init();

    let client = TestEcdsaClient::new();

    contracts_sanity::run_sanity_tests(&client).await;
}
