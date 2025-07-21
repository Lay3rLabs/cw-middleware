use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::{
    msg::InstantiateMsg,
    test::mocks::{
        mock_service_manager_contract_validate_success,
        mock_service_manager_contract_validation_error,
    },
};

pub const OWNER: &str = "owner";

fn wavs_counter_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub struct Suite {
    pub app: App,
    /// A service manager contract that validates successfully
    pub service_manager_success: Addr,
    /// A service manager contract that validates unsuccessfully
    pub service_manager_error: Addr,
    /// A wavs counter contract whose service manager validates successfully
    pub wavs_counter_success: Addr,
    /// A wavs counter contract whose service manager validates unsuccessfully
    pub wavs_counter_error: Addr,
}

impl Suite {
    pub fn new() -> Suite {
        let mut app = App::default();
        let service_manager_success_id =
            app.store_code(mock_service_manager_contract_validate_success());
        let service_manager_error_id =
            app.store_code(mock_service_manager_contract_validation_error());
        let wavs_counter_id = app.store_code(wavs_counter_contract());

        let service_manager_success = app
            .instantiate_contract(
                service_manager_success_id,
                Addr::unchecked(OWNER),
                &Empty {},
                &[],
                "service_manager",
                None,
            )
            .unwrap();

        let service_manager_error = app
            .instantiate_contract(
                service_manager_error_id,
                Addr::unchecked(OWNER),
                &Empty {},
                &[],
                "service_manager-error",
                None,
            )
            .unwrap();

        let wavs_counter_success = app
            .instantiate_contract(
                wavs_counter_id,
                Addr::unchecked(OWNER),
                &InstantiateMsg {
                    service_manager: service_manager_success.to_string(),
                },
                &[],
                "wavs-counter",
                None,
            )
            .unwrap();

        let wavs_counter_error = app
            .instantiate_contract(
                wavs_counter_id,
                Addr::unchecked(OWNER),
                &InstantiateMsg {
                    service_manager: service_manager_error.to_string(),
                },
                &[],
                "wavs-counter",
                None,
            )
            .unwrap();

        Suite {
            app,
            service_manager_success,
            service_manager_error,
            wavs_counter_success,
            wavs_counter_error,
        }
    }
}
