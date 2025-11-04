use cw_multi_test::{ContractWrapper, Executor};
use cw_wavs_sdk::contract_kinds::mirror::{
    MirrorServiceHandlerExecutor, MirrorServiceHandlerQuerier, MirrorServiceManagerExecutor,
    MirrorServiceManagerQuerier, MirrorStakeRegistryExecutor, MirrorStakeRegistryQuerier,
};
use cw_wavs_sdk::service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier};
use cw_wavs_sdk::service_manager::{ServiceManagerExecutor, ServiceManagerQuerier};
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

        // Service Manager (store code for stake registry)
        let service_manager_contract = ContractWrapper::new(
            cw_wavs_mirror_service_manager::entry::execute,
            cw_wavs_mirror_service_manager::entry::instantiate,
            cw_wavs_mirror_service_manager::entry::query,
        )
        .with_checksum(cosmwasm_std::Checksum::generate(b"mirror-service-manager"));

        let service_manager_code_id = app
            .borrow_mut()
            .store_code(Box::new(service_manager_contract));

        // Service Handler
        let service_handler_contract = ContractWrapper::new(
            cw_wavs_mirror_service_handler::entry::execute,
            cw_wavs_mirror_service_handler::entry::instantiate,
            cw_wavs_mirror_service_handler::entry::query,
        );
        let service_handler_code_id = app
            .borrow_mut()
            .store_code(Box::new(service_handler_contract));

        // Stake Registry
        let stake_registry_contract = ContractWrapper::new(
            cw_wavs_mirror_stake_registry::entry::execute,
            cw_wavs_mirror_stake_registry::entry::instantiate,
            cw_wavs_mirror_stake_registry::entry::query,
        );
        let stake_registry_code_id = app
            .borrow_mut()
            .store_code(Box::new(stake_registry_contract));

        // Create service manager instantiate message for stake registry
        let service_manager_instantiate_msg = cosmwasm_std::WasmMsg::Instantiate2 {
            admin: Some(admin.to_string()),
            code_id: service_manager_code_id,
            msg: cosmwasm_std::to_json_binary(
                &cw_wavs_mirror_api::service_manager::InstantiateMsg {
                    admin: admin.to_string(),
                },
            )
            .unwrap(),
            funds: vec![],
            label: "Mirror Service Manager".to_string(),
            salt: cosmwasm_std::Binary::from("service_manager".as_bytes()),
        };

        let stake_registry = app
            .borrow_mut()
            .instantiate_contract(
                stake_registry_code_id,
                admin.clone(),
                &cw_wavs_mirror_api::stake_registry::InstantiateMsg {
                    service_manager_instantiate: service_manager_instantiate_msg,
                    threshold_weight: cosmwasm_std::Uint256::from(1000u128),
                    quorum: cw_wavs_mirror_api::stake_registry::QuorumConfig {
                        strategies: vec![cw_wavs_mirror_api::stake_registry::StrategyParams {
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

        // Get the service manager address from stake registry
        let service_manager_addr: cosmwasm_std::Addr = app
            .borrow()
            .wrap()
            .query_wasm_smart(
                stake_registry.clone(),
                &cw_wavs_mirror_api::stake_registry::QueryMsg::GetServiceManager {},
            )
            .unwrap();

        // Now instantiate the service handler with the service manager address
        let service_handler = app
            .borrow_mut()
            .instantiate_contract(
                service_handler_code_id,
                admin.clone(),
                &cw_wavs_mirror_api::service_handler::InstantiateMsg {
                    service_manager: service_manager_addr.to_string(),
                },
                &[],
                "Mirror Service Handler",
                None,
            )
            .unwrap();

        // Create service clients
        let service_handler_querier = MirrorServiceHandlerQuerier::new(ServiceHandlerQuerier::new(
            client.querier.clone(),
            service_handler.clone(),
        ));
        let service_handler_executor = MirrorServiceHandlerExecutor::new(
            ServiceHandlerExecutor::new(client.executor.clone(), service_handler),
        );
        let service_manager_querier = MirrorServiceManagerQuerier::new(ServiceManagerQuerier::new(
            client.querier.clone(),
            service_manager_addr.clone(),
        ));
        let service_manager_executor = MirrorServiceManagerExecutor::new(
            ServiceManagerExecutor::new(client.executor.clone(), service_manager_addr),
        );
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

    pub fn wrap_test(
        &self,
        cw_wavs_trigger_simple: &crate::client::trigger::SimpleTriggerTestClient,
    ) -> ContractTestWrapper {
        ContractTestWrapper {
            service_handler_querier: self.service_handler_querier.service_handler().clone(),
            service_handler_executor: self.service_handler_executor.service_handler().clone(),
            service_manager_querier: self.service_manager_querier.service_manager().clone(),
            service_manager_executor: self.service_manager_executor.service_manager().clone(),
            cw_wavs_trigger_simple_querier: cw_wavs_trigger_simple.querier.clone(),
            cw_wavs_trigger_simple_executor: cw_wavs_trigger_simple.executor.clone(),
        }
    }
}
