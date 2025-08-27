use wavs_types::Service;

use crate::e2e::client::{TestClient, TestService};

impl TestClient {
    pub async fn register_operator(&self, service: &Service) {
        let signing_key_addr = self
            .node
            .get_service_signing_key_addr(service)
            .await
            .unwrap();

        let operator_addr = signing_key_addr.clone();
        tracing::info!("Adding weight for operator/avs-key {signing_key_addr} on mock contract");

        match &self.service {
            TestService::Mock(client) => {
                client
                    .service_manager_executor
                    .set_signing_key(operator_addr, signing_key_addr, 1)
                    .await
                    .unwrap();
            }
            TestService::Mirror(client) => {
                client
                    .service_manager_executor
                    .set_signing_key(operator_addr, signing_key_addr, 1)
                    .await
                    .unwrap();
            }
            TestService::Ecdsa(_) => {
                tracing::warn!(
                    "TODO - add weight for avs-key {} on ecdsa contract",
                    signing_key_addr
                );
            }
            TestService::Bls(_) => {
                tracing::warn!(
                    "TODO - add weight for avs-key {} on bls contract",
                    signing_key_addr
                );
            }
        }
    }
}
