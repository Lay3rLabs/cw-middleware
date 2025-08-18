use cosmwasm_std::Addr;
use off_chain_tests::client::{
    ecdsa::EcdsaTestClient, trigger::SimpleTriggerTestClient, ContractTestClient,
};
use shared_tests::{contracts_sanity, tracing_init::tracing_tests_init};

#[tokio::test]
async fn ecdsa_sanity() {
    tracing_tests_init();

    let client = ContractTestClient::new(Addr::unchecked("admin"));
    let ecdsa_client = EcdsaTestClient::new(client.clone());
    let trigger_client = SimpleTriggerTestClient::new(client);

    contracts_sanity::run_sanity_tests(&ecdsa_client.wrap_test(&trigger_client)).await;
}
