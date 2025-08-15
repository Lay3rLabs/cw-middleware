use async_trait::async_trait;
use bls_api::service_handler::QueryMsg;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use super::HasBlsServiceHandlerAddr;
use crate::ServiceHandlerQueryClient;

#[async_trait(?Send)]
pub trait BlsServiceHandlerQueryClient: ServiceHandlerQueryClient + HasBlsServiceHandlerAddr
{
    async fn bls_service_handler_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = HasBlsServiceHandlerAddr::addr(self);
        self.contract_query(&contract_addr, msg).await
    }
}

impl <T> BlsServiceHandlerQueryClient for T
where
    T: ServiceHandlerQueryClient + HasBlsServiceHandlerAddr { }