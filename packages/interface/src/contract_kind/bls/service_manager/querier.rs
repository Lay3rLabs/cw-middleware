use async_trait::async_trait;
use bls_api::service_manager::QueryMsg;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use super::HasBlsServiceManagerAddr;
use crate::ServiceManagerQueryClient;

#[async_trait(?Send)]
pub trait BlsServiceManagerQueryClient: ServiceManagerQueryClient + HasBlsServiceManagerAddr
{
    async fn bls_service_manager_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = HasBlsServiceManagerAddr::addr(self);
        self.contract_query(&contract_addr, msg).await
    }
}

impl <T> BlsServiceManagerQueryClient for T
where
    T: ServiceManagerQueryClient + HasBlsServiceManagerAddr { }