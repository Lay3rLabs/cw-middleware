#[cfg(feature = "multitest")]
use cw_multi_test::{App, Executor};
#[cfg(feature = "multitest")]
use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::Addr;
use layer_climb::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

#[derive(Clone)]
pub enum WavsQuerier {
    Climb(QueryClient),
    #[cfg(feature = "climb_pool")]
    ClimbPool(layer_climb::pool::SigningClientPool),
    #[cfg(feature = "multitest")]
    MultiTest(Rc<RefCell<App>>),
}

impl From<QueryClient> for WavsQuerier {
    fn from(client: QueryClient) -> WavsQuerier {
        WavsQuerier::Climb(client)
    }
}

#[cfg(feature = "climb_pool")]
impl From<layer_climb::pool::SigningClientPool> for WavsQuerier {
    fn from(pool: layer_climb::pool::SigningClientPool) -> WavsQuerier {
        WavsQuerier::ClimbPool(pool)
    }
}

#[cfg(feature = "multitest")]
impl From<Rc<RefCell<App>>> for WavsQuerier {
    fn from(app: Rc<RefCell<App>>) -> WavsQuerier {
        WavsQuerier::MultiTest(app)
    }
}

impl WavsQuerier {
    pub async fn contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        match self {
            Self::Climb(client) => {
                let addr = layer_climb::prelude::Address::try_from(address)
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                client
                    .contract_smart(&addr, msg)
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
            }
            #[cfg(feature = "climb_pool")]
            Self::ClimbPool(pool) => {
                let addr = layer_climb::prelude::Address::try_from(address)
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                let client = pool
                    .get()
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                client
                    .querier
                    .contract_smart(&addr, msg)
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
            }
            #[cfg(feature = "multitest")]
            Self::MultiTest(app) => app.borrow().wrap().query_wasm_smart(address, msg),
        }
    }
}

#[derive(Clone)]
pub enum WavsExecutor {
    Climb(SigningClient),
    #[cfg(feature = "climb_pool")]
    ClimbPool(layer_climb::pool::SigningClientPool),
    #[cfg(feature = "multitest")]
    MultiTest {
        app: Rc<RefCell<App>>,
        admin: Addr,
    },
}

impl From<SigningClient> for WavsExecutor {
    fn from(client: SigningClient) -> WavsExecutor {
        WavsExecutor::Climb(client)
    }
}

#[cfg(feature = "climb_pool")]
impl From<layer_climb::pool::SigningClientPool> for WavsExecutor {
    fn from(pool: layer_climb::pool::SigningClientPool) -> WavsExecutor {
        WavsExecutor::ClimbPool(pool)
    }
}

#[cfg(feature = "multitest")]
impl From<(Rc<RefCell<App>>, Addr)> for WavsExecutor {
    fn from((app, admin): (Rc<RefCell<App>>, Addr)) -> WavsExecutor {
        WavsExecutor::MultiTest { app, admin }
    }
}

impl WavsExecutor {
    pub async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        match self {
            Self::Climb(client) => {
                let addr = Address::try_from(address)
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                let funds = funds
                    .iter()
                    .map(|c| layer_climb::prelude::Coin {
                        denom: c.denom.clone(),
                        amount: c.amount.to_string(),
                    })
                    .collect::<Vec<_>>();

                client
                    .contract_execute(&addr, msg, funds, None)
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
                    .map(WavsTxResponse::Climb)
            }
            #[cfg(feature = "climb_pool")]
            Self::ClimbPool(pool) => {
                let client = pool
                    .get()
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                let addr = Address::try_from(address)
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                let funds = funds
                    .iter()
                    .map(|c| layer_climb::prelude::Coin {
                        denom: c.denom.clone(),
                        amount: c.amount.to_string(),
                    })
                    .collect::<Vec<_>>();

                client
                    .contract_execute(&addr, msg, funds, None)
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
                    .map(WavsTxResponse::Climb)
            }
            #[cfg(feature = "multitest")]
            Self::MultiTest { app, admin } => app
                .borrow_mut()
                .execute_contract(admin.clone(), address.clone(), msg, funds)
                .map(WavsTxResponse::MultiTest),
        }
    }
}

pub enum WavsTxResponse {
    Climb(layer_climb::proto::abci::TxResponse),
    #[cfg(feature = "multitest")]
    MultiTest(cw_multi_test::AppResponse),
}

impl<'a> From<&'a WavsTxResponse> for CosmosTxEvents<'a> {
    fn from(value: &'a WavsTxResponse) -> Self {
        match value {
            WavsTxResponse::Climb(resp) => CosmosTxEvents::from(resp),
            #[cfg(feature = "multitest")]
            WavsTxResponse::MultiTest(resp) => CosmosTxEvents::from(resp.events.as_slice()),
        }
    }
}

impl WavsTxResponse {
    pub fn unchecked_into_tx_response(self) -> layer_climb::proto::abci::TxResponse {
        match self {
            Self::Climb(tx_resp) => tx_resp,
            _ => panic!("unable to get unchecked tx response"),
        }
    }
}
