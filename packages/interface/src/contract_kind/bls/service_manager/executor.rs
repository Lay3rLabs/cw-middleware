use async_trait::async_trait;
use cosmwasm_std::Coin;
use bls_api::service_manager::ExecuteMsg;
use super::HasBlsServiceManagerAddr;
use crate::ServiceManagerExecClient;


#[async_trait(?Send)]
pub trait BlsServiceManagerExecClient: ServiceManagerExecClient + HasBlsServiceManagerAddr
{
    async fn bls_service_manager_exec(
        &self,
        msg: &ExecuteMsg,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = HasBlsServiceManagerAddr::addr(self);
        self.contract_exec(&contract_addr, msg, funds).await
    }
}

impl <T> BlsServiceManagerExecClient for T
where
    T: ServiceManagerExecClient + HasBlsServiceManagerAddr { }