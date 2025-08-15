use crate::e2e::{
    client::TestClient,
    operator::register_operator,
    service::{activate_service, deploy_service},
};

pub mod client;
mod operator;
mod runner;
mod service;

impl TestClient {
    pub async fn run(self) {
        let mut service = deploy_service(&self).await;
        register_operator(&self, &service).await;
        activate_service(&self, &mut service).await;
        runner::run_e2e_tests(self.clone()).await;
        tracing::info!("E2E tests completed successfully.");
    }
}
