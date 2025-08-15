use async_trait::async_trait;
use cosmwasm_std::{Addr, Coin};
use serde::Serialize;

use crate::{QueryClientExt, TxResponseExt};

#[async_trait(?Send)]
pub trait ExecClientExt: QueryClientExt {
    type TxResponse: TxResponseExt;

    async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError>;
}
