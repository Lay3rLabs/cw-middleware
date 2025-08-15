use interface::*;


pub async fn run_sanity_tests(client: &impl ServiceTriggerClient){
    run_sanity_tests_with_id(client, "").await;
}
pub async fn run_sanity_tests_with_id(client: &impl ServiceTriggerClient, id: &str) {
    // Example sanity test
    let addr = client
        .handler()
        .get_manager_address()
        .await
        .unwrap();
    tracing::info!("[{id}] Service Handler Manager Address: {}", addr);

    client
        .manager()
        .set_service_uri("http://example.com".to_string())
        .await
        .unwrap();

    let url = client
        .manager()
        .get_service_uri()
        .await
        .unwrap();

    assert_eq!(url, "http://example.com");

    let trigger_id = client
        .trigger()
        .push_message("hello world")
        .await
        .unwrap();

    assert!(trigger_id.u64() > 0, "Trigger ID should be greater than 0");

    let trigger_message = client
        .trigger()
        .get_trigger_message(trigger_id)
        .await
        .unwrap();

    assert_eq!(trigger_message, "hello world");
}
