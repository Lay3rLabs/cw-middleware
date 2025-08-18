use cosmwasm_std::Addr;
use off_chain_tests::client::{
    mock::MockTestClient, trigger::SimpleTriggerTestClient, ContractTestClient,
};
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn mock_sanity() {
    tracing_tests_init();

    let client = ContractTestClient::new(Addr::unchecked("admin"));
    let mock_client = MockTestClient::new(client.clone());
    let trigger_client = SimpleTriggerTestClient::new(client);

    contracts_sanity::run_sanity_tests(&mock_client.wrap_test(&trigger_client)).await;
}

#[tokio::test]
async fn mock_handler_works() {
    tracing_tests_init();

    let client = ContractTestClient::new(Addr::unchecked("admin"));
    let mock_client = MockTestClient::new(client.clone());

    mock_client
        .service_handler_executor
        .set_trigger_message(42u64.into(), "hello world")
        .await
        .unwrap();

    let msg = mock_client
        .service_handler_querier
        .get_handled_trigger_message(42u64.into())
        .await
        .unwrap();

    assert_eq!(msg, "hello world");
}
