use on_chain_tests::client::contract::new_mock_client;
use shared_tests::{sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn mock_sanity() {
    tracing_tests_init();

    let client = new_mock_client().await;

    sanity::run_sanity_tests(&client).await;
}
