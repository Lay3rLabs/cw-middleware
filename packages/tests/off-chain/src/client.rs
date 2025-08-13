use std::{cell::RefCell, rc::Rc};

use async_trait::async_trait;
use cosmwasm_std::{Addr, Coin, Empty};
use cw_multi_test::{App, ContractWrapper, Executor};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use utils::prelude::*;

#[derive(Clone)]
pub struct TestClient {
    pub app: Rc<RefCell<App>>,
    pub handler: TestServiceHandlerClient,
    pub manager: TestServiceManagerClient,
    pub admin: Addr,
}

impl Default for TestClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TestClient {
    pub fn new() -> Self {
        let admin = Addr::unchecked("admin");
        let app = Rc::new(RefCell::new(App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &admin,
                    vec![Coin {
                        denom: "utoken".to_string(),
                        amount: 1_000_000u128.into(),
                    }],
                )
                .unwrap();
        })));

        let manager = TestServiceManagerClient::new(app.clone(), admin.clone());
        let handler = TestServiceHandlerClient::new(app.clone(), &manager.addr, admin.clone());

        Self {
            app,
            handler,
            manager,
            admin,
        }
    }
}

#[derive(Clone)]
pub struct TestServiceHandlerClient {
    pub app: Rc<RefCell<App>>,
    pub code_id: u64,
    pub addr: Addr,
    pub admin: Addr,
}

impl TestServiceHandlerClient {
    pub fn new(app: Rc<RefCell<App>>, manager_addr: &Addr, admin: Addr) -> Self {
        // Create contract wrappers
        let contract = ContractWrapper::new(
            mock_service_handler::entry::execute,
            mock_service_handler::entry::instantiate,
            mock_service_handler::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin.clone(),
                &mock_api::service_handler::InstantiateMsg {
                    service_manager: manager_addr.to_string(),
                },
                &[],
                "Service Handler",
                None,
            )
            .unwrap();

        Self {
            app,
            code_id,
            addr,
            admin,
        }
    }
}

#[derive(Clone)]
pub struct TestServiceManagerClient {
    pub app: Rc<RefCell<App>>,
    pub code_id: u64,
    pub addr: Addr,
    pub admin: Addr,
}

impl TestServiceManagerClient {
    pub fn new(app: Rc<RefCell<App>>, admin: Addr) -> Self {
        // Create contract wrappers
        let contract = ContractWrapper::new(
            mock_service_manager::entry::execute,
            mock_service_manager::entry::instantiate,
            mock_service_manager::entry::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let addr = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin.clone(),
                &Empty {},
                &[],
                "Service Manager",
                None,
            )
            .unwrap();

        Self {
            app,
            code_id,
            addr,
            admin,
        }
    }
}

// Service Handler impls
#[async_trait(?Send)]
impl WavsBasicQueryClientExt for TestServiceHandlerClient {
    async fn basic_contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.app.borrow().wrap().query_wasm_smart(address, msg)
    }
}

impl WavsServiceHandlerAddrExt for TestServiceHandlerClient {
    fn addr(&self) -> Addr {
        self.addr.clone()
    }
}

#[async_trait(?Send)]
impl WavsBasicExecClientExt for TestServiceHandlerClient {
    type TxResponse = cw_multi_test::AppResponse;

    async fn basic_contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        self.app
            .borrow_mut()
            .execute_contract(self.admin.clone(), address.clone(), msg, funds)
    }
}

// Service Manager impls
#[async_trait(?Send)]
impl WavsBasicQueryClientExt for TestServiceManagerClient {
    async fn basic_contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.app.borrow().wrap().query_wasm_smart(address, msg)
    }
}

#[async_trait(?Send)]
impl WavsBasicExecClientExt for TestServiceManagerClient {
    type TxResponse = cw_multi_test::AppResponse;

    async fn basic_contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        self.app
            .borrow_mut()
            .execute_contract(self.admin.clone(), address.clone(), msg, funds)
    }
}

impl WavsServiceManagerAddrExt for TestServiceManagerClient {
    fn addr(&self) -> Addr {
        self.addr.clone()
    }
}

// Combined impls
impl WavsQueryClientExt for TestClient {
    type ServiceHandler = TestServiceHandlerClient;
    type ServiceManager = TestServiceManagerClient;

    fn service_handler(&self) -> &Self::ServiceHandler {
        &self.handler
    }
    fn service_manager(&self) -> &Self::ServiceManager {
        &self.manager
    }
}

impl WavsExecClientExt for TestClient {
    type ServiceHandler = TestServiceHandlerClient;
    type ServiceManager = TestServiceManagerClient;

    fn service_handler(&self) -> &Self::ServiceHandler {
        &self.handler
    }

    fn service_manager(&self) -> &Self::ServiceManager {
        &self.manager
    }
}
