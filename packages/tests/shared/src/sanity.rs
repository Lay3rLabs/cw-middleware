use utils::prelude::*;

pub async fn run_sanity_tests(client: &impl WavsClientExt) {
    // Example sanity test
    let addr = client
        .service_handler_querier()
        .get_manager_address()
        .await
        .unwrap();
    tracing::info!("Service Handler Manager Address: {}", addr);

    client
        .service_manager_exec()
        .set_service_uri("http://example.com".to_string())
        .await
        .unwrap();
    let url = client
        .service_manager_querier()
        .get_service_uri()
        .await
        .unwrap();

    assert_eq!(url, "http://example.com");
}
