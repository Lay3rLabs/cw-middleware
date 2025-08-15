use on_chain_tests::e2e::client::TestClient;
use shared_tests::tracing_init::tracing_tests_init;

#[tokio::test]
async fn bls_e2e() {
    tracing_tests_init();

    TestClient::new_bls().await.run().await;
}
