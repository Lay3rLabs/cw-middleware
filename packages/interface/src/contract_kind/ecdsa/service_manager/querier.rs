use async_trait::async_trait;
use ecdsa_api::service_manager::QueryMsg;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use super::HasEcdsaServiceManagerAddr;
use crate::ServiceManagerQueryClient;

#[async_trait(?Send)]
pub trait EcdsaServiceManagerQueryClient: ServiceManagerQueryClient + HasEcdsaServiceManagerAddr
{
    async fn ecdsa_service_manager_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = HasEcdsaServiceManagerAddr::addr(self);
        self.contract_query(&contract_addr, msg).await
    }
}

impl <T> EcdsaServiceManagerQueryClient for T
where
    T: ServiceManagerQueryClient + HasEcdsaServiceManagerAddr { }