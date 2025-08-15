use async_trait::async_trait;
use cosmwasm_std::Addr;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

#[async_trait(?Send)]
pub trait QueryClientExt {
    async fn contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError>;
}