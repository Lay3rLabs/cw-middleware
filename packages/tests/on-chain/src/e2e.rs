use std::time::Duration;

use cosmwasm_std::Uint64;

use crate::{
    client::contract::mock::MockTestClient,
    e2e::client::{TestClient, TestService},
};

pub mod client;
mod operator;
mod service;

impl TestClient {
    pub async fn run(self) {
        let mut service = self.deploy_service().await;
        self.register_operator(&service).await;
        self.activate_service(&mut service).await;

        tracing::info!("Sending trigger");
        let trigger_id = self
            .trigger
            .executor
            .push_message("hello world!".as_bytes().to_vec())
            .await
            .unwrap();

        match &self.service {
            TestService::Mock(mock_contract_client) => {
                handle_mock_response(mock_contract_client, trigger_id).await;
            }
            _ => {
                tracing::warn!("E2E tests are currently only implemented for MockContractClient");
            }
        }

        tracing::info!("E2E tests completed successfully.");
    }
}

async fn handle_mock_response(client: &MockTestClient, trigger_id: Uint64) {
    tokio::time::timeout(Duration::from_secs(30), async move {
        loop {
            match client
                .service_handler_querier
                .get_handled_trigger_message(trigger_id)
                .await
            {
                Ok(s) => {
                    let s = std::str::from_utf8(&s).unwrap();
                    tracing::info!("Received trigger message for trigger {trigger_id}: {s}");
                    assert_eq!(s, "hello world!");
                    break;
                }
                Err(_) => {
                    tracing::warn!("Waiting for response to land for trigger {trigger_id}...");
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap()
}
