use async_trait::async_trait;
use serde::de::DeserializeOwned;
use wavs_types::contracts::cosmwasm::service_manager::ServiceManagerQueryMessages;
use std::fmt::Debug;

use crate::{HasServiceManagerAddr, QueryClientExt};


#[async_trait(?Send)]
pub trait ServiceManagerQueryClient:
    QueryClientExt + HasServiceManagerAddr
{
    async fn service_manager_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &ServiceManagerQueryMessages,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = self.addr();
        self.contract_query(&contract_addr, msg).await
    }

    async fn get_service_uri(&self) -> Result<String, cosmwasm_std::StdError> {
        self.service_manager_query(&ServiceManagerQueryMessages::WavsServiceUri {})
            .await
    }
}

impl <T> ServiceManagerQueryClient for T
where
    T: QueryClientExt + HasServiceManagerAddr { }