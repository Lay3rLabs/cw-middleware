use crate::e2e::client::TestClient;

pub mod client;
mod operator;
mod runner;
mod service;

impl TestClient {
    pub async fn run(self) {
        service::deploy_service(self.clone()).await;
        runner::run_e2e_tests(self.clone()).await;
        tracing::info!("E2E tests completed successfully.");
    }
}
