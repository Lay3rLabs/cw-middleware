use cw_multi_test::{ContractWrapper, Executor};
use sdk::contract_kinds::mirror::{MirrorStakeRegistryExecutor, MirrorStakeRegistryQuerier};

use crate::client::ContractTestClient;

#[derive(Clone)]
pub struct MirrorTestClient {
    pub stake_registry_querier: MirrorStakeRegistryQuerier,
    pub stake_registry_executor: MirrorStakeRegistryExecutor,
}

impl MirrorTestClient {
    pub fn new(client: ContractTestClient) -> Self {
        let app = client.app();
        let admin = client.admin();

        let contract = ContractWrapper::new(
            mirror_stake_registry::entry::execute,
            mirror_stake_registry::entry::instantiate,
            mirror_stake_registry::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let stake_registry = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin.clone(),
                &mirror_api::stake_registry::InstantiateMsg {
                    service_manager: "test_service_manager".to_string(),
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

        let stake_registry_querier =
            MirrorStakeRegistryQuerier::new(client.querier.clone(), stake_registry.clone());
        let stake_registry_executor =
            MirrorStakeRegistryExecutor::new(client.executor, stake_registry);

        Self {
            stake_registry_querier,
            stake_registry_executor,
        }
    }
}
