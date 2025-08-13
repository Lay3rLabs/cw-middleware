use shared_tests::tracing_init::tracing_tests_init;

#[tokio::test]
async fn test_full() {
    tracing_tests_init();
    tracing::info!("Running full end-to-end test suite");
}
