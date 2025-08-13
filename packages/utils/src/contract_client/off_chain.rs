use std::{cell::RefCell, fmt::Debug, rc::Rc};

use async_trait::async_trait;
use cosmwasm_std::{Addr, Coin};
use cw_multi_test::{App, Executor};
use serde::{de::DeserializeOwned, Serialize};

use crate::prelude::{
    WavsBasicExecClientExt, WavsBasicQueryClientExt, WavsExecClientExt, WavsQueryClientExt,
    WavsServiceHandlerAddrExt, WavsServiceManagerAddrExt, WavsTriggerAddrExt,
};

#[derive(Clone)]
pub struct WavsApp {
    pub inner: Rc<RefCell<App>>,
    pub admin: Addr,
    service_handler_addr: Addr,
    service_manager_addr: Addr,
    trigger_addr: Addr,
}

impl WavsApp {
    pub fn new(
        inner: Rc<RefCell<App>>,
        admin: Addr,
        service_handler_addr: Addr,
        service_manager_addr: Addr,
        trigger_addr: Addr,
    ) -> Self {
        Self {
            inner,
            admin,
            service_handler_addr,
            service_manager_addr,
            trigger_addr,
        }
    }
}

#[async_trait(?Send)]
impl WavsBasicQueryClientExt for WavsApp {
    async fn basic_contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.inner.borrow().wrap().query_wasm_smart(address, msg)
    }
}

#[async_trait(?Send)]
impl WavsBasicExecClientExt for WavsApp {
    type TxResponse = cw_multi_test::AppResponse;

    async fn basic_contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        self.inner
            .borrow_mut()
            .execute_contract(self.admin.clone(), address.clone(), msg, funds)
    }
}

impl WavsServiceManagerAddrExt for WavsApp {
    fn addr(&self) -> Addr {
        self.service_manager_addr.clone()
    }
}

impl WavsServiceHandlerAddrExt for WavsApp {
    fn addr(&self) -> Addr {
        self.service_handler_addr.clone()
    }
}

impl WavsTriggerAddrExt for WavsApp {
    fn addr(&self) -> Addr {
        self.trigger_addr.clone()
    }
}

// Combined impls
impl WavsQueryClientExt for WavsApp {
    type ServiceHandler = WavsApp;
    type ServiceManager = WavsApp;
    type Trigger = WavsApp;

    fn service_handler(&self) -> &Self::ServiceHandler {
        &self
    }
    fn service_manager(&self) -> &Self::ServiceManager {
        &self
    }
    fn trigger(&self) -> &Self::Trigger {
        &self
    }
}

impl WavsExecClientExt for WavsApp {
    type ServiceHandler = WavsApp;
    type ServiceManager = WavsApp;
    type Trigger = WavsApp;

    fn service_handler(&self) -> &Self::ServiceHandler {
        &self
    }

    fn service_manager(&self) -> &Self::ServiceManager {
        &self
    }

    fn trigger(&self) -> &Self::ServiceManager {
        &self
    }
}
