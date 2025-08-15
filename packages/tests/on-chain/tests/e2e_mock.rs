use on_chain_tests::e2e::{client::TestClient, runner::run_e2e_tests};
use shared_tests::tracing_init::tracing_tests_init;

#[tokio::test]
async fn mock_e2e() {
    tracing_tests_init();

    let client = TestClient::new_mock().await;
    run_e2e_tests(client).await;
}
