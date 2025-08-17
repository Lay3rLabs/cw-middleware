use futures::{stream::FuturesUnordered, StreamExt};
use on_chain_tests::client::contract::{
    mock::MockTestClient, simple_trigger::SimpleTriggerTestClient, ContractTestClient,
};
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test(flavor = "multi_thread")]
async fn mock_sanity_1() {
    tracing_tests_init();

    let mut futures = FuturesUnordered::new();

    for _ in 0..5 {
        futures.push(async {
            let client = ContractTestClient::new().await;
            let simple_trigger = SimpleTriggerTestClient::new(client.clone()).await;
            let contract = MockTestClient::new(client.clone()).await;
            let wrapped = contract.wrap_test(&simple_trigger);
            contracts_sanity::run_sanity_tests_with_id(&wrapped, "1").await;
        });
    }

    while (futures.next().await).is_some() {}
}

#[tokio::test(flavor = "multi_thread")]
async fn mock_sanity_2() {
    tracing_tests_init();

    let mut futures = FuturesUnordered::new();

    for _ in 0..5 {
        futures.push(async {
            let client = ContractTestClient::new().await;
            let simple_trigger = SimpleTriggerTestClient::new(client.clone()).await;
            let contract = MockTestClient::new(client.clone()).await;
            let wrapped = contract.wrap_test(&simple_trigger);
            contracts_sanity::run_sanity_tests_with_id(&wrapped, "2").await;
        });
    }

    while (futures.next().await).is_some() {}
}

#[tokio::test(flavor = "multi_thread")]
async fn mock_sanity_3() {
    tracing_tests_init();

    let mut futures = FuturesUnordered::new();

    for _ in 0..5 {
        futures.push(async {
            let client = ContractTestClient::new().await;
            let simple_trigger = SimpleTriggerTestClient::new(client.clone()).await;
            let contract = MockTestClient::new(client.clone()).await;
            let wrapped = contract.wrap_test(&simple_trigger);
            contracts_sanity::run_sanity_tests_with_id(&wrapped, "3").await;
        });
    }

    while (futures.next().await).is_some() {}
}
