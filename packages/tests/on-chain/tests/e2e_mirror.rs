use on_chain_tests::e2e::client::TestClient;
use shared_tests::tracing_init::tracing_tests_init;

#[tokio::test]
async fn mirror_e2e() {
    tracing_tests_init();

    TestClient::new_mirror().await.run().await;
}
