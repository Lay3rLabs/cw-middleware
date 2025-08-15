use async_trait::async_trait;
use ecdsa_api::service_handler::QueryMsg;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use super::HasEcdsaServiceHandlerAddr;

use crate::ServiceHandlerQueryClient;

#[async_trait(?Send)]
pub trait EcdsaServiceHandlerQueryClient: ServiceHandlerQueryClient + HasEcdsaServiceHandlerAddr
{
    async fn ecdsa_service_handler_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = HasEcdsaServiceHandlerAddr::addr(self);
        self.contract_query(&contract_addr, msg).await
    }
}

impl <T> EcdsaServiceHandlerQueryClient for T
where
    T: ServiceHandlerQueryClient + HasEcdsaServiceHandlerAddr { }