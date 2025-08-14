use on_chain_tests::{client::TestClient, e2e::run_e2e_tests};
use shared_tests::{tracing_init::tracing_tests_init};

#[tokio::test]
async fn ecdsa_e2e() {
    tracing_tests_init();

    let client = TestClient::new_ecdsa().await;
    run_e2e_tests(client).await;
}