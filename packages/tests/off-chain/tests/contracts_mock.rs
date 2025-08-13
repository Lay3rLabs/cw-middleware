use off_chain_tests::client::TestMockClient;
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn mock_sanity() {
    tracing_tests_init();

    let client = TestMockClient::new();

    contracts_sanity::run_sanity_tests(&client).await;
}
