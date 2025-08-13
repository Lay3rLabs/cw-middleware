use off_chain_tests::client::TestClient;
use shared_tests::{sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn mock_sanity() {
    tracing_tests_init();

    let client = TestClient::new();

    sanity::run_sanity_tests(&client).await;
}
