use on_chain_tests::client::TestClientBuilder;
use shared_tests::{tracing_init::tracing_tests_init};
use wavs_types::{Service, ServiceStatus};

#[tokio::test]
async fn mock_e2e() {
    tracing_tests_init();

    let client = TestClientBuilder::new()
        .with_service_maker(|_contracts, service_manager| async move {
            Service {
                name: "TestService".to_string(),
                status: ServiceStatus::Paused,
                workflows: Default::default(),
                manager: service_manager,
            }
        })
        .build()
        .await;

    // Use the client for testing
    println!("Service created: {}", client.service.name);
}