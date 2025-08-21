use cosmwasm_std::Addr;
use off_chain_tests::client::{
    mirror::MirrorTestClient, trigger::SimpleTriggerTestClient, ContractTestClient,
};
use shared_tests::{contracts_sanity, mirror_stake_registry, tracing_init::tracing_tests_init};

#[tokio::test]
async fn mirror_sanity() {
    tracing_tests_init();

    let client = ContractTestClient::new(Addr::unchecked("admin"));
    let mirror_client = MirrorTestClient::new(client.clone());
    let trigger_client = SimpleTriggerTestClient::new(client);

    contracts_sanity::run_sanity_tests(&mirror_client.wrap_test(&trigger_client)).await;
}

#[tokio::test]
async fn test_mirror_operator_management() {
    tracing_tests_init();

    let client = ContractTestClient::new(Addr::unchecked("admin"));
    let mirror_client = MirrorTestClient::new(client);

    mirror_stake_registry::run_mirror_operator_management_test(
        &mirror_client.stake_registry_executor,
        &mirror_client.stake_registry_querier,
    )
    .await;
}

#[tokio::test]
async fn test_mirror_batch_operator_management() {
    tracing_tests_init();

    let client = ContractTestClient::new(Addr::unchecked("admin"));
    let mirror_client = MirrorTestClient::new(client);

    mirror_stake_registry::run_mirror_batch_operator_management_test(
        &mirror_client.stake_registry_executor,
        &mirror_client.stake_registry_querier,
    )
    .await;
}

#[tokio::test]
async fn test_mirror_abi_signature_validation() {
    tracing_tests_init();

    let client = ContractTestClient::new(Addr::unchecked("admin"));
    let mirror_client = MirrorTestClient::new(client);

    mirror_stake_registry::run_mirror_abi_signature_validation_test(
        &mirror_client.stake_registry_executor,
        &mirror_client.stake_registry_querier,
    )
    .await;
}

#[tokio::test]
async fn test_mirror_abi_binary_compatibility() {
    tracing_tests_init();

    mirror_stake_registry::run_mirror_abi_binary_compatibility_test().await;
}

#[tokio::test]
async fn test_mirror_service_manager_admin_only() {
    tracing_tests_init();

    let admin = Addr::unchecked("admin");
    let non_admin = Addr::unchecked("not_admin");

    let client = ContractTestClient::new(admin.clone());
    let mirror_client = MirrorTestClient::new(client.clone());

    // Admin should be able to set signing key
    let operator = layer_climb_address::AddrEvm::new([0x01; 20]);
    let signing_key = layer_climb_address::AddrEvm::new([0x02; 20]);
    let weight = cosmwasm_std::Uint256::from(100u64);

    let result = mirror_client
        .service_manager_executor
        .mirror_exec(
            &mirror_api::service_manager::ExecuteMsg::SetSigningKey {
                operator: operator.clone(),
                signing_key: signing_key.clone(),
                weight,
            },
            &[],
        )
        .await;

    assert!(result.is_ok(), "Admin should be able to set signing key");

    // Non-admin trying to set signing key should fail
    // Create non-admin executor but use same service manager contract
    let non_admin_client = ContractTestClient::new(non_admin);
    let non_admin_executor = sdk::service_manager::ServiceManagerExecutor::new(
        non_admin_client.executor,
        mirror_client.service_manager_executor.service_manager().addr.clone(),
    );
    let non_admin_mirror_executor = sdk::contract_kinds::mirror::MirrorServiceManagerExecutor::new(non_admin_executor);

    let result = non_admin_mirror_executor
        .mirror_exec(
            &mirror_api::service_manager::ExecuteMsg::SetSigningKey {
                operator: operator.clone(),
                signing_key: signing_key.clone(),
                weight,
            },
            &[],
        )
        .await;

    assert!(
        result.is_err(),
        "Non-admin should not be able to set signing key"
    );
}
