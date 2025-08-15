use futures::{stream::FuturesUnordered, StreamExt};
use on_chain_tests::client::TestClient;
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test(flavor = "multi_thread")]
async fn mock_sanity_1() {
    tracing_tests_init();

    let mut futures = FuturesUnordered::new();

    for _ in 0..5 {
        futures.push(async {
            let client = TestClient::new_mock().await;
            contracts_sanity::run_sanity_tests_with_id(&client, "1").await;
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
            let client = TestClient::new_mock().await;
            contracts_sanity::run_sanity_tests_with_id(&client, "2").await;
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
            let client = TestClient::new_mock().await;
            contracts_sanity::run_sanity_tests_with_id(&client, "3").await;
        });
    }

    while (futures.next().await).is_some() {}
}
