use std::{cell::RefCell, rc::Rc, fmt::Debug};

use async_trait::async_trait;
use cosmwasm_std::{Addr, Coin};
use cw_multi_test::{App, Executor};
use crate::{ExecClientExt, HasServiceHandlerAddr, HasServiceManagerAddr, HasSimpleTriggerAddr, QueryClientExt, TxResponseExt};
use layer_climb::events::CosmosTxEvents;
use serde::{de::DeserializeOwned, Serialize};

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
impl QueryClientExt for WavsApp {
    async fn contract_query<
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
impl ExecClientExt for WavsApp {
    type TxResponse = cw_multi_test::AppResponse;

    async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
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

impl TxResponseExt for cw_multi_test::AppResponse {
    fn extract_events(&self) -> CosmosTxEvents<'_> {
        CosmosTxEvents::from(self.events.as_slice())
    }
}

impl HasServiceManagerAddr for WavsApp {
    fn addr(&self) -> Addr {
        self.service_manager_addr.clone()
    }
}

impl HasServiceHandlerAddr for WavsApp {
    fn addr(&self) -> Addr {
        self.service_handler_addr.clone()
    }
}

impl HasSimpleTriggerAddr for WavsApp {
    fn addr(&self) -> Addr {
        self.trigger_addr.clone()
    }
}