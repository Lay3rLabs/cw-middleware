use on_chain_tests::client::contract::EcdsaContractClient;
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn mock_sanity() {
    tracing_tests_init();

    let client = EcdsaContractClient::new().await;

    contracts_sanity::run_sanity_tests(&client).await;
}
