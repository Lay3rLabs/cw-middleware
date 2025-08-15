use async_trait::async_trait;
use cosmwasm_std::Coin;
use ecdsa_api::service_manager::ExecuteMsg;
use super::HasEcdsaServiceManagerAddr;
use crate::ServiceManagerExecClient;


#[async_trait(?Send)]
pub trait EcdsaServiceManagerExecClient: ServiceManagerExecClient + HasEcdsaServiceManagerAddr
{
    async fn ecdsa_service_manager_exec(
        &self,
        msg: &ExecuteMsg,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = HasEcdsaServiceManagerAddr::addr(self);
        self.contract_exec(&contract_addr, msg, funds).await
    }
}

impl <T> EcdsaServiceManagerExecClient for T
where
    T: ServiceManagerExecClient + HasEcdsaServiceManagerAddr { }