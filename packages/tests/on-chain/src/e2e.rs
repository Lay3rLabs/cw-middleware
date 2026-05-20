use crate::e2e::client::TestClient;

pub mod client;
mod operator;
mod service;

impl TestClient {
    pub async fn run(self) {
        let mut service = self.deploy_service().await;
        self.register_operator(&service).await;
        self.activate_service(&mut service).await;

        tracing::info!("Sending trigger");
        let _trigger_id = self
            .trigger
            .executor
            .push_message("hello world!".as_bytes().to_vec())
            .await
            .unwrap();

        // Per-flavor response polling will be re-introduced alongside the
        // ECDSA/BLS/mirror service-handler implementations (Phase 2/3 of the
        // 2026-05-09 audit fix plan).
        tracing::warn!("E2E response polling not yet wired for any service flavor");

        tracing::info!("E2E tests completed (deploy + register + activate paths).");
    }
}
