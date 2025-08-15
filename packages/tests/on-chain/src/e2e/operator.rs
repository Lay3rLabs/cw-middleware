use wavs_types::Service;

use crate::e2e::client::{TestClient, TestContractClient};

pub async fn register_operator(client: &TestClient, service: &Service) {
    let signing_key_addr = client
        .node
        .get_service_signing_key_addr(service)
        .await
        .unwrap();

    match &client.contract {
        TestContractClient::Mock(mock_contract_client) => {
            let operator_addr = signing_key_addr.clone();
            tracing::info!(
                "Adding weight for operator/avs-key {signing_key_addr} on mock contract"
            );
            mock_contract_client
                .mock_service_manager_set_signing_key(operator_addr, signing_key_addr, 1)
                .await
                .unwrap();
        }
        TestContractClient::Ecdsa(_) => {
            tracing::warn!(
                "TODO - add weight for avs-key {} on ecdsa contract",
                signing_key_addr
            );
        }
        TestContractClient::Bls(_) => {
            tracing::warn!(
                "TODO - add weight for avs-key {} on bls contract",
                signing_key_addr
            );
        }
    }
}
