use std::time::Duration;

use cosmwasm_std::Uint64;
use utils::{contract_client::functionality::WavsTriggerExecClientExt, prelude::*};
use wavs_types::ServiceStatus;

use crate::e2e::client::{TestClient, TestContractClient};

pub async fn run_e2e_tests(client: TestClient) {
    let mut service = client.new_service().await;

    service.status = ServiceStatus::Active;

    let service_url = client.node.save_service_url(&service).await.unwrap();

    tracing::info!("Service URL: {}", service_url);

    client
        .service_manager_exec()
        .set_service_uri(service_url)
        .await
        .unwrap();

    client
        .node
        .deploy_service(service.manager.clone())
        .await
        .unwrap();
    client
        .node
        .register_aggregator_service(&service)
        .await
        .unwrap();

    let trigger_id = client
        .trigger_exec()
        .push_message("hello world!")
        .await
        .unwrap();

    match client.contract {
        TestContractClient::Mock(mock_contract_client) => {
            handle_mock_response(mock_contract_client, trigger_id).await;
        }
        _ => {
            tracing::warn!("E2E tests are currently only implemented for MockContractClient");
        }
    }
}

async fn handle_mock_response(client: impl WavsMockQueryClientExt, trigger_id: Uint64) {
    tokio::time::timeout(Duration::from_secs(5), async move {
        loop {
            match client.get_service_handler_trigger_message(trigger_id).await {
                Ok(s) => {
                    assert_eq!(s, "hello world!");
                }
                Err(_) => {
                    tracing::warn!("Waiting for response to land...");
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap()
}
