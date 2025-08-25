use sdk::contract_kinds::mirror::{
    MirrorServiceHandlerExecutor, MirrorServiceHandlerQuerier, MirrorServiceManagerExecutor,
    MirrorServiceManagerQuerier, MirrorStakeRegistryExecutor, MirrorStakeRegistryQuerier,
};
use sdk::service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier};
use sdk::service_manager::{ServiceManagerExecutor, ServiceManagerQuerier};
use shared_tests::wrapper::ContractTestWrapper;

use crate::client::code_ids::CodeId;
use crate::client::contract::trigger::SimpleTriggerTestClient;
use crate::client::contract::ContractTestClient;

#[derive(Clone)]
pub struct MirrorTestClient {
    pub service_handler_querier: MirrorServiceHandlerQuerier,
    pub service_handler_executor: MirrorServiceHandlerExecutor,
    pub service_manager_querier: MirrorServiceManagerQuerier,
    pub service_manager_executor: MirrorServiceManagerExecutor,
    pub stake_registry_querier: MirrorStakeRegistryQuerier,
    pub stake_registry_executor: MirrorStakeRegistryExecutor,
}

impl MirrorTestClient {
    pub async fn new(test_client: ContractTestClient) -> Self {
        let pool = test_client.pool();
        let client = pool.get().await.unwrap();
        let admin = client.addr.clone();

        // Create service manager instantiate message for stake registry
        let service_manager_code_id = CodeId::new_mirror_service_manager().await;
        let service_manager_instantiate_msg = cosmwasm_std::WasmMsg::Instantiate2 {
            admin: Some(admin.to_string()),
            code_id: service_manager_code_id,
            msg: cosmwasm_std::to_json_binary(&mirror_api::service_manager::InstantiateMsg {
                owner: admin.to_string(),
            })
            .unwrap(),
            funds: vec![],
            label: "Mirror Service Manager".to_string(),
            salt: cosmwasm_std::Binary::from(b"service_manager"),
        };

        let (stake_registry, _) = client
            .contract_instantiate(
                None,
                CodeId::new_mirror_stake_registry().await,
                "Mirror Stake Registry",
                &mirror_api::stake_registry::InstantiateMsg {
                    service_manager_instantiate: service_manager_instantiate_msg,
                    threshold_weight: cosmwasm_std::Uint256::from(1000u128),
                    quorum: mirror_api::stake_registry::QuorumConfig {
                        strategies: vec![mirror_api::stake_registry::StrategyParams {
                            strategy: "test_strategy".to_string(),
                            multiplier: cosmwasm_std::Uint256::from(100u128),
                        }],
                    },
                },
                vec![],
                None,
            )
            .await
            .unwrap();

        // Get the service manager address from stake registry
        let service_manager_addr: cosmwasm_std::Addr = test_client
            .querier
            .contract_query(
                &stake_registry.clone().try_into().unwrap(),
                &mirror_api::stake_registry::QueryMsg::GetServiceManager {},
            )
            .await
            .unwrap();

        // Now instantiate the service handler with the service manager address
        let (service_handler, _) = client
            .contract_instantiate(
                None,
                CodeId::new_mirror_service_handler().await,
                "Mirror Service Handler",
                &mirror_api::service_handler::InstantiateMsg {
                    service_manager: service_manager_addr.to_string(),
                },
                vec![],
                None,
            )
            .await
            .unwrap();

        let service_handler_querier = MirrorServiceHandlerQuerier::new(ServiceHandlerQuerier::new(
            test_client.querier.clone(),
            service_handler.clone().try_into().unwrap(),
        ));
        let service_handler_executor =
            MirrorServiceHandlerExecutor::new(ServiceHandlerExecutor::new(
                test_client.executor.clone(),
                service_handler.try_into().unwrap(),
            ));
        let service_manager_querier = MirrorServiceManagerQuerier::new(ServiceManagerQuerier::new(
            test_client.querier.clone(),
            service_manager_addr.clone(),
        ));
        let service_manager_executor = MirrorServiceManagerExecutor::new(
            ServiceManagerExecutor::new(test_client.executor.clone(), service_manager_addr),
        );
        let stake_registry_querier = MirrorStakeRegistryQuerier::new(
            test_client.querier.clone(),
            stake_registry.clone().try_into().unwrap(),
        );
        let stake_registry_executor = MirrorStakeRegistryExecutor::new(
            test_client.executor.clone(),
            stake_registry.try_into().unwrap(),
        );

        Self {
            service_handler_querier,
            service_handler_executor,
            service_manager_querier,
            service_manager_executor,
            stake_registry_querier,
            stake_registry_executor,
        }
    }

    pub fn wrap_test(&self, trigger_simple: &SimpleTriggerTestClient) -> ContractTestWrapper {
        ContractTestWrapper {
            service_handler_querier: self.service_handler_querier.service_handler().clone(),
            service_handler_executor: self.service_handler_executor.service_handler().clone(),
            service_manager_querier: self.service_manager_querier.service_manager().clone(),
            service_manager_executor: self.service_manager_executor.service_manager().clone(),
            trigger_simple_querier: trigger_simple.querier.clone(),
            trigger_simple_executor: trigger_simple.executor.clone(),
        }
    }
}
