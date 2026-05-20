pub mod bls;
pub mod ecdsa;
pub mod mirror;
pub mod trigger;

use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::{Addr, Coin};
use cw_multi_test::App;
use cw_wavs_sdk::client::{WavsExecutor, WavsQuerier};

#[derive(Clone)]
pub struct ContractTestClient {
    pub querier: WavsQuerier,
    pub executor: WavsExecutor,
}

impl ContractTestClient {
    pub fn new(admin: &str) -> Self {
        let app = Rc::new(RefCell::new(App::new(|router, api, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &api.addr_make(admin),
                    vec![Coin {
                        denom: "utoken".to_string(),
                        amount: 1_000_000u128.into(),
                    }],
                )
                .unwrap();
        })));

        let admin = app.borrow().api().addr_make(admin);

        Self {
            querier: app.clone().into(),
            executor: (app.clone(), admin).into(),
        }
    }

    pub fn app(&self) -> Rc<RefCell<App>> {
        match &self.executor {
            WavsExecutor::MultiTest { app, .. } => app.clone(),
            _ => unreachable!(),
        }
    }

    pub fn admin(&self) -> Addr {
        match &self.executor {
            WavsExecutor::MultiTest { admin, .. } => admin.clone(),
            _ => unreachable!(),
        }
    }
}
