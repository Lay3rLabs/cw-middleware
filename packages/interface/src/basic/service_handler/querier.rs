use async_trait::async_trait;
use cosmwasm_std::Addr;
use serde::de::DeserializeOwned;
use wavs_types::contracts::cosmwasm::service_handler::ServiceHandlerQueryMessages;
use std::fmt::Debug;

use crate::{HasServiceHandlerAddr, QueryClientExt};


#[async_trait(?Send)]
pub trait ServiceHandlerQueryClient: 
    QueryClientExt + HasServiceHandlerAddr
{
    async fn service_handler_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &ServiceHandlerQueryMessages,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = self.addr();
        self.contract_query(&contract_addr, msg).await
    }

    async fn get_manager_address(&self) -> Result<Addr, cosmwasm_std::StdError> {
        self.service_handler_query(&ServiceHandlerQueryMessages::WavsServiceManager {})
            .await
    }
}

impl <T> ServiceHandlerQueryClient for T
where
    T: QueryClientExt + HasServiceHandlerAddr { }