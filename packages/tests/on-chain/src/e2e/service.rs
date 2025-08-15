use std::collections::BTreeMap;

use wavs_types::{
    AllowedHostPermission, Component, ComponentSource, CosmosContractSubmission, Service,
    ServiceManager, Submit, Trigger, Workflow,
};

use crate::{
    client::{config::TestConfig, node::WavsNodeClient},
    e2e::client::TestClient,
};

pub async fn deploy_service(client: &TestClient) -> Service {
    let service = new_service(client).await;

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

    service
}

pub async fn activate_service(client: &TestClient, service: &mut Service) {
    service.status = wavs_types::ServiceStatus::Active;

    let service_url = client.node.save_service_url(service).await.unwrap();

    client
        .service_manager_exec()
        .set_service_uri(service_url)
        .await
        .unwrap();
}

async fn new_service(client: &TestClient) -> Service {
    let component = WavsNodeClient::component_digest().await;
    let mut component = Component::new(ComponentSource::Digest(component));
    component.permissions.allowed_http_hosts = AllowedHostPermission::All;

    let mut workflows = BTreeMap::new();

    workflows.insert(
        "messenger".parse().unwrap(),
        Workflow {
            trigger: Trigger::CosmosContractEvent {
                address: client.trigger_address(),
                chain_name: client.config.chain_name.clone(),
                event_type: PushMessageEvent::EVENT_TYPE.to_string(),
            },
            component,
            submit: Submit::Aggregator {
                url: TestConfig::aggregator_endpoint(),
                component: None,
                evm_contracts: None,
                cosmos_contracts: Some(vec![CosmosContractSubmission::new(
                    client.config.chain_name.clone(),
                    client.service_handler_address(),
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
            chain_name: client.config.chain_name.clone(),
            address: client.service_manager_address(),
        },
    }
}
