use async_trait::async_trait;
use cosmwasm_std::Coin;
use ecdsa_api::service_handler::ExecuteMsg;
use super::HasEcdsaServiceHandlerAddr;
use crate::ServiceHandlerExecClient;


#[async_trait(?Send)]
pub trait EcdsaServiceHandlerExecClient: ServiceHandlerExecClient + HasEcdsaServiceHandlerAddr
{
    async fn ecdsa_service_handler_exec(
        &self,
        msg: &ExecuteMsg,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = HasEcdsaServiceHandlerAddr::addr(self);
        self.contract_exec(&contract_addr, msg, funds).await
    }
}

impl <T> EcdsaServiceHandlerExecClient for T
where
    T: ServiceHandlerExecClient + HasEcdsaServiceHandlerAddr { }