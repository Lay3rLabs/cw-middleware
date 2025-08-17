use std::collections::BTreeMap;
use trigger_api::simple::PushMessageEvent;
use wavs_types::{
    AllowedHostPermission, Component, ComponentSource, CosmosContractSubmission, Service,
    ServiceManager, Submit, Trigger, Workflow,
};

use crate::{
    client::{config::TestConfig, node::WavsNodeClient},
    e2e::client::TestClient,
};

impl TestClient {
    pub async fn deploy_service(&self) -> Service {
        let service = self.new_service().await;

        let service_url = self.node.save_service_url(&service).await.unwrap();

        tracing::info!("Service URL: {}", service_url);

        self.service
            .wavs_service_manager_executor()
            .set_service_uri(service_url)
            .await
            .unwrap();

        self.node
            .deploy_service(service.manager.clone())
            .await
            .unwrap();

        self.node
            .register_aggregator_service(&service)
            .await
            .unwrap();

        service
    }

    pub async fn activate_service(&self, service: &mut Service) {
        service.status = wavs_types::ServiceStatus::Active;

        let service_url = self.node.save_service_url(service).await.unwrap();

        self.service
            .wavs_service_manager_executor()
            .set_service_uri(service_url)
            .await
            .unwrap();
    }

    async fn new_service(&self) -> Service {
        let component = WavsNodeClient::component_digest().await;
        let mut component = Component::new(ComponentSource::Digest(component));
        component.permissions.allowed_http_hosts = AllowedHostPermission::All;

        let mut workflows = BTreeMap::new();

        workflows.insert(
            "messenger".parse().unwrap(),
            Workflow {
                trigger: Trigger::CosmosContractEvent {
                    address: self.trigger.querier.addr.clone().try_into().unwrap(),
                    chain_name: self.config.chain_name.clone(),
                    event_type: PushMessageEvent::EVENT_TYPE.to_string(),
                },
                component,
                submit: Submit::Aggregator {
                    url: TestConfig::aggregator_endpoint(),
                    component: None,
                    evm_contracts: None,
                    cosmos_contracts: Some(vec![CosmosContractSubmission::new(
                        self.config.chain_name.clone(),
                        self.service
                            .wavs_service_handler_querier()
                            .addr
                            .try_into()
                            .unwrap(),
                        None,
                    )]),
                },
            },
        );

        Service {
            name: "test".to_string(),
            status: wavs_types::ServiceStatus::Paused,
            workflows,
            manager: ServiceManager::Cosmos {
                chain_name: self.config.chain_name.clone(),
                address: self
                    .service
                    .wavs_service_manager_querier()
                    .addr
                    .try_into()
                    .unwrap(),
            },
        }
    }
}
