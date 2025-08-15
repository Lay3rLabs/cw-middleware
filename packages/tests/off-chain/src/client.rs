use std::{cell::RefCell, rc::Rc, sync::LazyLock};

use async_trait::async_trait;
use cosmwasm_std::{Addr, Coin};
use cw_multi_test::{App, ContractWrapper, Executor};
use utils::{contract_client::off_chain::WavsApp, prelude::*};

static ADMIN: LazyLock<Addr> = LazyLock::new(|| Addr::unchecked("admin"));

#[derive(Clone)]
pub struct TestMockClient {
    pub app: WavsApp,
}

impl Default for TestMockClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TestMockClient {
    pub fn new() -> Self {
        let app = Rc::new(RefCell::new(App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &ADMIN,
                    vec![Coin {
                        denom: "utoken".to_string(),
                        amount: 1_000_000u128.into(),
                    }],
                )
                .unwrap();
        })));

        // Create contract wrappers
        let contract = ContractWrapper::new(
            mock_service_manager::entry::execute,
            mock_service_manager::entry::instantiate,
            mock_service_manager::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let manager_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &mock_api::service_manager::InstantiateMsg {},
                &[],
                "Mock Service Manager",
                None,
            )
            .unwrap();

        let contract = ContractWrapper::new(
            mock_service_handler::entry::execute,
            mock_service_handler::entry::instantiate,
            mock_service_handler::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let handler_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &mock_api::service_handler::InstantiateMsg {
                    service_manager: manager_addr.to_string(),
                },
                &[],
                "Mock Service Handler",
                None,
            )
            .unwrap();

        let contract = ContractWrapper::new(
            mock_trigger::entry::execute,
            mock_trigger::entry::instantiate,
            mock_trigger::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let trigger_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &mock_api::trigger::InstantiateMsg {},
                &[],
                "Mock Trigger",
                None,
            )
            .unwrap();

        let app = WavsApp::new(
            app,
            Addr::unchecked("admin"),
            Addr::unchecked(handler_addr),
            Addr::unchecked(manager_addr),
            Addr::unchecked(trigger_addr),
        );

        Self { app }
    }
}

#[derive(Clone)]
pub struct TestEcdsaClient {
    pub app: WavsApp,
}

impl Default for TestEcdsaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEcdsaClient {
    pub fn new() -> Self {
        let app = Rc::new(RefCell::new(App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &ADMIN,
                    vec![Coin {
                        denom: "utoken".to_string(),
                        amount: 1_000_000u128.into(),
                    }],
                )
                .unwrap();
        })));

        // Create contract wrappers
        let contract = ContractWrapper::new(
            ecdsa_service_manager::entry::execute,
            ecdsa_service_manager::entry::instantiate,
            ecdsa_service_manager::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let manager_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &ecdsa_api::service_manager::InstantiateMsg {},
                &[],
                "ECDSA Service Manager",
                None,
            )
            .unwrap();

        let contract = ContractWrapper::new(
            ecdsa_service_handler::entry::execute,
            ecdsa_service_handler::entry::instantiate,
            ecdsa_service_handler::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let handler_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &ecdsa_api::service_handler::InstantiateMsg {
                    service_manager: manager_addr.to_string(),
                },
                &[],
                "ECDSA Service Handler",
                None,
            )
            .unwrap();

        let contract = ContractWrapper::new(
            mock_trigger::entry::execute,
            mock_trigger::entry::instantiate,
            mock_trigger::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let trigger_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &mock_api::trigger::InstantiateMsg {},
                &[],
                "Mock Trigger",
                None,
            )
            .unwrap();

        let app = WavsApp::new(
            app,
            Addr::unchecked("admin"),
            Addr::unchecked(handler_addr),
            Addr::unchecked(manager_addr),
            Addr::unchecked(trigger_addr),
        );

        Self { app }
    }
}

#[derive(Clone)]
pub struct TestBlsClient {
    pub app: WavsApp,
}

impl Default for TestBlsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TestBlsClient {
    pub fn new() -> Self {
        let app = Rc::new(RefCell::new(App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &ADMIN,
                    vec![Coin {
                        denom: "utoken".to_string(),
                        amount: 1_000_000u128.into(),
                    }],
                )
                .unwrap();
        })));

        // Create contract wrappers
        let contract = ContractWrapper::new(
            bls_service_manager::entry::execute,
            bls_service_manager::entry::instantiate,
            bls_service_manager::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let manager_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &bls_api::service_manager::InstantiateMsg {},
                &[],
                "BLS Service Manager",
                None,
            )
            .unwrap();

        let contract = ContractWrapper::new(
            bls_service_handler::entry::execute,
            bls_service_handler::entry::instantiate,
            bls_service_handler::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let handler_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &bls_api::service_handler::InstantiateMsg {
                    service_manager: manager_addr.to_string(),
                },
                &[],
                "BLS Service Handler",
                None,
            )
            .unwrap();

        let contract = ContractWrapper::new(
            mock_trigger::entry::execute,
            mock_trigger::entry::instantiate,
            mock_trigger::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let trigger_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &mock_api::trigger::InstantiateMsg {},
                &[],
                "Mock Trigger",
                None,
            )
            .unwrap();

        let app = WavsApp::new(
            app,
            Addr::unchecked("admin"),
            Addr::unchecked(handler_addr),
            Addr::unchecked(manager_addr),
            Addr::unchecked(trigger_addr),
        );

        Self { app }
    }
}

impl HasWavsQueryClient for TestMockClient {
    type QueryClient = WavsApp;

    fn query_client(&self) -> &Self::QueryClient {
        &self.app
    }
}

impl HasWavsExecClient for TestMockClient {
    type ExecClient = WavsApp;

    fn exec_client(&self) -> &Self::ExecClient {
        &self.app
    }
}

#[async_trait(?Send)]
impl WavsMockQueryClientExt for TestMockClient {}
#[async_trait(?Send)]
impl WavsMockExecClientExt for TestMockClient {}

impl HasWavsQueryClient for TestEcdsaClient {
    type QueryClient = WavsApp;

    fn query_client(&self) -> &Self::QueryClient {
        &self.app
    }
}

impl HasWavsExecClient for TestEcdsaClient {
    type ExecClient = WavsApp;

    fn exec_client(&self) -> &Self::ExecClient {
        &self.app
    }
}

#[async_trait(?Send)]
impl WavsEcdsaQueryClientExt for TestEcdsaClient {}
#[async_trait(?Send)]
impl WavsEcdsaExecClientExt for TestEcdsaClient {}

impl HasWavsQueryClient for TestBlsClient {
    type QueryClient = WavsApp;

    fn query_client(&self) -> &Self::QueryClient {
        &self.app
    }
}

impl HasWavsExecClient for TestBlsClient {
    type ExecClient = WavsApp;

    fn exec_client(&self) -> &Self::ExecClient {
        &self.app
    }
}

#[async_trait(?Send)]
impl WavsBlsQueryClientExt for TestBlsClient {}
#[async_trait(?Send)]
impl WavsBlsExecClientExt for TestBlsClient {}
