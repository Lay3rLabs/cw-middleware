use cw_wavs_trigger_api::simple::PushMessageEvent;
use std::collections::BTreeMap;
use wavs_types_full::{
    AllowedHostPermission, Component, ComponentSource, Service, ServiceManager, ServiceStatus,
    SignatureKind, Submit, Trigger, Workflow,
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
        service.status = wavs_types_full::ServiceStatus::Active;

        let service_url = self.node.save_service_url(service).await.unwrap();

        self.service
            .wavs_service_manager_executor()
            .set_service_uri(service_url)
            .await
            .unwrap();
    }

    async fn new_service(&self) -> Service {
        let operator_component = WavsNodeClient::operator_component_digest().await;
        let mut operator_component = Component::new(ComponentSource::Digest(operator_component));
        operator_component.permissions.allowed_http_hosts = AllowedHostPermission::All;

        let aggregator_component = WavsNodeClient::aggregator_component_digest().await;
        let mut aggregator_component =
            Component::new(ComponentSource::Digest(aggregator_component));
        aggregator_component.permissions.allowed_http_hosts = AllowedHostPermission::All;

        let mut workflows = BTreeMap::new();

        // evm_contracts: None,
        // cosmos_contracts: Some(vec![CosmosContractSubmission::new(
        //     self.config.chain_name.clone(),
        //     self.service
        //         .wavs_service_handler_querier()
        //         .addr
        //         .try_into()
        //         .unwrap(),
        //     None,
        // )]),

        workflows.insert(
            "messenger".parse().unwrap(),
            Workflow {
                trigger: Trigger::CosmosContractEvent {
                    address: self.trigger.querier.addr.clone().try_into().unwrap(),
                    chain: self.config.chain.clone(),
                    event_type: PushMessageEvent::EVENT_TYPE.to_string(),
                },
                component: operator_component,
                submit: Submit::Aggregator {
                    url: TestConfig::aggregator_endpoint(),
                    component: Box::new(aggregator_component),
                    signature_kind: SignatureKind::evm_default(),
                },
            },
        );

        Service {
            name: "test".to_string(),
            status: ServiceStatus::Paused,
            workflows,
            manager: ServiceManager::Cosmos {
                chain: self.config.chain.clone(),
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
