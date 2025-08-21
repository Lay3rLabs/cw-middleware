use cw_multi_test::{ContractWrapper, Executor};
use sdk::contract_kinds::mirror::{
    MirrorServiceHandlerExecutor, MirrorServiceHandlerQuerier, MirrorServiceManagerExecutor,
    MirrorServiceManagerQuerier, MirrorStakeRegistryExecutor, MirrorStakeRegistryQuerier,
};
use sdk::service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier};
use sdk::service_manager::{ServiceManagerExecutor, ServiceManagerQuerier};
use shared_tests::wrapper::ContractTestWrapper;

use crate::client::ContractTestClient;

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
    pub fn new(client: ContractTestClient) -> Self {
        let app = client.app();
        let admin = client.admin();

        // Service Manager
        let service_manager_contract = ContractWrapper::new(
            mirror_service_manager::entry::execute,
            mirror_service_manager::entry::instantiate,
            mirror_service_manager::entry::query,
        );
        let service_manager_code_id = app.borrow_mut().store_code(Box::new(service_manager_contract));
        let service_manager = app
            .borrow_mut()
            .instantiate_contract(
                service_manager_code_id,
                admin.clone(),
                &mirror_api::service_manager::InstantiateMsg {
                    admin: admin.to_string(),
                },
                &[],
                "Mirror Service Manager",
                None,
            )
            .unwrap();

        // Service Handler
        let service_handler_contract = ContractWrapper::new(
            mirror_service_handler::entry::execute,
            mirror_service_handler::entry::instantiate,
            mirror_service_handler::entry::query,
        );
        let service_handler_code_id = app.borrow_mut().store_code(Box::new(service_handler_contract));
        let service_handler = app
            .borrow_mut()
            .instantiate_contract(
                service_handler_code_id,
                admin.clone(),
                &mirror_api::service_handler::InstantiateMsg {
                    admin: admin.to_string(),
                    service_manager: service_manager.to_string(),
                },
                &[],
                "Mirror Service Handler",
                None,
            )
            .unwrap();

        // Stake Registry
        let stake_registry_contract = ContractWrapper::new(
            mirror_stake_registry::entry::execute,
            mirror_stake_registry::entry::instantiate,
            mirror_stake_registry::entry::query,
        );
        let stake_registry_code_id = app.borrow_mut().store_code(Box::new(stake_registry_contract));
        let stake_registry = app
            .borrow_mut()
            .instantiate_contract(
                stake_registry_code_id,
                admin.clone(),
                &mirror_api::stake_registry::InstantiateMsg {
                    service_manager: service_manager.to_string(),
                    threshold_weight: cosmwasm_std::Uint256::from(1000u128),
                    quorum: mirror_api::stake_registry::QuorumConfig {
                        strategies: vec![mirror_api::stake_registry::StrategyParams {
                            strategy: "test_strategy".to_string(),
                            multiplier: cosmwasm_std::Uint256::from(100u128),
                        }],
                    },
                },
                &[],
                "Mirror Stake Registry",
                None,
            )
            .unwrap();

        // Create service clients
        let service_handler_querier = MirrorServiceHandlerQuerier::new(ServiceHandlerQuerier::new(
            client.querier.clone(),
            service_handler.clone(),
        ));
        let service_handler_executor = MirrorServiceHandlerExecutor::new(ServiceHandlerExecutor::new(
            client.executor.clone(),
            service_handler,
        ));
        let service_manager_querier = MirrorServiceManagerQuerier::new(ServiceManagerQuerier::new(
            client.querier.clone(),
            service_manager.clone(),
        ));
        let service_manager_executor = MirrorServiceManagerExecutor::new(ServiceManagerExecutor::new(
            client.executor.clone(),
            service_manager,
        ));
        let stake_registry_querier =
            MirrorStakeRegistryQuerier::new(client.querier.clone(), stake_registry.clone());
        let stake_registry_executor =
            MirrorStakeRegistryExecutor::new(client.executor, stake_registry);

        Self {
            service_handler_querier,
            service_handler_executor,
            service_manager_querier,
            service_manager_executor,
            stake_registry_querier,
            stake_registry_executor,
        }
    }

    pub fn wrap_test(&self, trigger_simple: &crate::client::trigger::SimpleTriggerTestClient) -> ContractTestWrapper {
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
