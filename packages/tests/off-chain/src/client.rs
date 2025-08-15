use std::{cell::RefCell, rc::Rc, sync::LazyLock};
use std::fmt::Debug;
use async_trait::async_trait;
use cosmwasm_std::{Addr, Coin};
use cw_multi_test::{App, ContractWrapper, Executor};
use interface::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

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
            trigger_simple::entry::execute,
            trigger_simple::entry::instantiate,
            trigger_simple::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let trigger_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &trigger_api::simple::InstantiateMsg {},
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
            trigger_simple::entry::execute,
            trigger_simple::entry::instantiate,
            trigger_simple::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let trigger_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &trigger_api::simple::InstantiateMsg {},
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
            trigger_simple::entry::execute,
            trigger_simple::entry::instantiate,
            trigger_simple::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let trigger_addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                ADMIN.clone(),
                &trigger_api::simple::InstantiateMsg {},
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

// Mock
#[async_trait(?Send)]
impl QueryClientExt for TestMockClient {
    async fn contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.app.contract_query(address, msg).await
    }
}

#[async_trait(?Send)]
impl ExecClientExt for TestMockClient {
    type TxResponse = cw_multi_test::AppResponse;

    async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        self.app.contract_exec(address, msg, funds).await
    }
}

impl HasServiceHandlerAddr for TestMockClient {
    fn addr(&self) -> Addr {
        HasServiceHandlerAddr::addr(&self.app)
    }
}

impl HasServiceManagerAddr for TestMockClient {
    fn addr(&self) -> Addr {
        HasServiceManagerAddr::addr(&self.app)
    }
}

impl HasSimpleTriggerAddr for TestMockClient {
    fn addr(&self) -> Addr {
        HasSimpleTriggerAddr::addr(&self.app)
    }
}

impl HasMockServiceHandlerAddr for TestMockClient {
    fn addr(&self) -> Addr {
        HasServiceHandlerAddr::addr(&self.app)
    }
}
impl HasMockServiceManagerAddr for TestMockClient {
    fn addr(&self) -> Addr {
        HasServiceManagerAddr::addr(&self.app)
    }
}

// Ecdsa
#[async_trait(?Send)]
impl QueryClientExt for TestEcdsaClient {
    async fn contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.app.contract_query(address, msg).await
    }
}

#[async_trait(?Send)]
impl ExecClientExt for TestEcdsaClient {
    type TxResponse = cw_multi_test::AppResponse;

    async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        self.app.contract_exec(address, msg, funds).await
    }
}

impl HasServiceHandlerAddr for TestEcdsaClient {
    fn addr(&self) -> Addr {
        HasServiceHandlerAddr::addr(&self.app)
    }
}

impl HasServiceManagerAddr for TestEcdsaClient {
    fn addr(&self) -> Addr {
        HasServiceManagerAddr::addr(&self.app)
    }
}

impl HasSimpleTriggerAddr for TestEcdsaClient {
    fn addr(&self) -> Addr {
        HasSimpleTriggerAddr::addr(&self.app)
    }
}
impl HasEcdsaServiceHandlerAddr for TestEcdsaClient {
    fn addr(&self) -> Addr {
        HasServiceHandlerAddr::addr(&self.app)
    }
}
impl HasEcdsaServiceManagerAddr for TestEcdsaClient {
    fn addr(&self) -> Addr {
        HasServiceManagerAddr::addr(&self.app)
    }
}

// BLS
#[async_trait(?Send)]
impl QueryClientExt for TestBlsClient {
    async fn contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.app.contract_query(address, msg).await
    }
}

#[async_trait(?Send)]
impl ExecClientExt for TestBlsClient {
    type TxResponse = cw_multi_test::AppResponse;

    async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        self.app.contract_exec(address, msg, funds).await
    }
}

impl HasBlsServiceHandlerAddr for TestBlsClient {
    fn addr(&self) -> Addr {
        HasServiceHandlerAddr::addr(&self.app)
    }
}
impl HasBlsServiceManagerAddr for TestBlsClient {
    fn addr(&self) -> Addr {
        HasServiceManagerAddr::addr(&self.app)
    }
}

impl HasServiceHandlerAddr for TestBlsClient {
    fn addr(&self) -> Addr {
        HasServiceHandlerAddr::addr(&self.app)
    }
}

impl HasServiceManagerAddr for TestBlsClient {
    fn addr(&self) -> Addr {
        HasServiceManagerAddr::addr(&self.app)
    }
}

impl HasSimpleTriggerAddr for TestBlsClient {
    fn addr(&self) -> Addr {
        HasSimpleTriggerAddr::addr(&self.app)
    }
}
