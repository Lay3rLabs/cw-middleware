use off_chain_tests::client::TestMockClient;
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};
use utils::prelude::*;

#[tokio::test]
async fn mock_sanity() {
    tracing_tests_init();

    let client = TestMockClient::new();

    contracts_sanity::run_sanity_tests(&client).await;
}

#[tokio::test]
async fn mock_handler_works() {
    tracing_tests_init();

    let client = TestMockClient::new();

    client
        .mock_service_handler_set_trigger_message(42u64.into(), "hello world")
        .await
        .unwrap();

    let msg = client
        .get_service_handler_trigger_message(42u64.into())
        .await
        .unwrap();

    assert_eq!(msg, "hello world");
}
