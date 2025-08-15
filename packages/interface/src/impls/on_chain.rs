mod test_app;
pub use test_app::*;

use std::fmt::Debug;
use async_trait::async_trait;
use cosmwasm_std::Addr;
use layer_climb::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::{ExecClientExt, QueryClientExt, TxResponseExt};

#[async_trait(?Send)]
impl QueryClientExt for QueryClient {
    async fn contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let addr = Address::try_from(address).map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
        self.contract_smart(&addr, msg).await.map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
    }
}

#[async_trait(?Send)]
impl QueryClientExt for SigningClient {
    async fn contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.querier.contract_query(address, msg)
            .await
            .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
    }
}

#[async_trait(?Send)]
impl ExecClientExt for SigningClient {
    type TxResponse = layer_climb::proto::abci::TxResponse;

    async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let addr = Address::try_from(address).map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;

        let funds = funds
            .iter()
            .map(|c| layer_climb::prelude::Coin {
                denom: c.denom.clone(),
                amount: c.amount.to_string(),
            })
            .collect::<Vec<_>>();

        self
            .contract_execute(&addr, msg, funds, None)
            .await
            .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))

    }
}


impl TxResponseExt for layer_climb::proto::abci::TxResponse {
    fn extract_events(&self) -> CosmosTxEvents<'_> {
        CosmosTxEvents::from(self)
    }
}

#[cfg(feature = "impl_on_chain_pool")]
pub mod pool {
    use std::fmt::Debug;
    use async_trait::async_trait;
    use cosmwasm_std::Addr;
    use deadpool::managed::Object;
    use layer_climb::prelude::*;
    use serde::{de::DeserializeOwned, Serialize};

    use crate::{ExecClientExt, QueryClientExt};


    #[async_trait(?Send)]
    impl QueryClientExt for Object<SigningClientPoolManager> {
        async fn contract_query<
            RESP: DeserializeOwned + Send + Sync + Debug,
            MSG: Serialize + Debug,
        >(
            &self,
            address: &Addr,
            msg: &MSG,
        ) -> Result<RESP, cosmwasm_std::StdError> {
            let addr = Address::try_from(address).map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
            self.querier.contract_smart(&addr, msg).await.map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
        }
    }

    #[async_trait(?Send)]
    impl ExecClientExt for Object<SigningClientPoolManager> {
        type TxResponse = layer_climb::proto::abci::TxResponse;

        async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
            &self,
            address: &Addr,
            msg: &MSG,
            funds: &[cosmwasm_std::Coin],
        ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
                    let addr = Address::try_from(address).map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;

        let funds = funds
            .iter()
            .map(|c| layer_climb::prelude::Coin {
                denom: c.denom.clone(),
                amount: c.amount.to_string(),
            })
            .collect::<Vec<_>>();

        self
            .contract_execute(&addr, msg, funds, None)
            .await
            .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
        }
    }

        #[async_trait(?Send)]
    impl QueryClientExt for SigningClientPool {
        async fn contract_query<
            RESP: DeserializeOwned + Send + Sync + Debug,
            MSG: Serialize + Debug,
        >(
            &self,
            address: &Addr,
            msg: &MSG,
        ) -> Result<RESP, cosmwasm_std::StdError> {
            let client = self.get().await.map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;

            client.contract_query(address, msg).await
        }
    }

    #[async_trait(?Send)]
    impl ExecClientExt for SigningClientPool {
        type TxResponse = layer_climb::proto::abci::TxResponse;

        async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
            &self,
            address: &Addr,
            msg: &MSG,
            funds: &[cosmwasm_std::Coin],
        ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
            let client = self.get().await.map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
            client.contract_exec(address, msg, funds).await
        }
    }
}