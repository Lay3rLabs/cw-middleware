use crate::wrapper::ContractTestWrapper;

pub async fn run_sanity_tests(contracts: &ContractTestWrapper) {
    run_sanity_tests_with_id(contracts, "").await;
}
pub async fn run_sanity_tests_with_id(contracts: &ContractTestWrapper, id: &str) {
    // Example sanity test
    let manager_addr = contracts
        .service_handler_querier
        .get_manager_address()
        .await
        .unwrap();
    tracing::info!("[{id}] Service Handler Manager Address: {}", manager_addr);

    contracts
        .service_manager_executor
        .set_service_uri("http://example.com".to_string())
        .await
        .unwrap();

    let url = contracts
        .service_manager_querier
        .get_service_uri()
        .await
        .unwrap();

    assert_eq!(url, "http://example.com");

    let trigger_id = contracts
        .cw_wavs_trigger_simple_executor
        .push_message("hello world".as_bytes().to_vec())
        .await
        .unwrap();

    assert!(trigger_id.u64() > 0, "Trigger ID should be greater than 0");

    let trigger_message = contracts
        .cw_wavs_trigger_simple_querier
        .get_trigger_message(trigger_id)
        .await
        .unwrap();

    assert_eq!(str::from_utf8(&trigger_message).unwrap(), "hello world");
}
