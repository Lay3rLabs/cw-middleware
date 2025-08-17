pub mod bls;
pub mod ecdsa;
pub mod mock;
pub mod simple_trigger;

use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::{Addr, Coin};
use cw_multi_test::App;
use sdk::client::{WavsExecutor, WavsQuerier};

#[derive(Clone)]
pub struct ContractTestClient {
    pub querier: WavsQuerier,
    pub executor: WavsExecutor,
}

impl ContractTestClient {
    pub fn new(admin: Addr) -> Self {
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
