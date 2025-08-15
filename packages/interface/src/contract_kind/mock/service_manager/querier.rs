use async_trait::async_trait;
use mock_api::service_manager::QueryMsg;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use super::HasMockServiceManagerAddr;
use crate::ServiceManagerQueryClient;

#[async_trait(?Send)]
pub trait MockServiceManagerQueryClient: ServiceManagerQueryClient + HasMockServiceManagerAddr
{
    async fn mock_service_manager_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let contract_addr = HasMockServiceManagerAddr::addr(self);
        self.contract_query(&contract_addr, msg).await
    }
}

impl <T> MockServiceManagerQueryClient for T
where
    T: ServiceManagerQueryClient + HasMockServiceManagerAddr { }