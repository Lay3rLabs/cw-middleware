use async_trait::async_trait;
use cosmwasm_std::Coin;
use bls_api::service_handler::ExecuteMsg;
use super::HasBlsServiceHandlerAddr;

use crate::ServiceHandlerExecClient;


#[async_trait(?Send)]
pub trait BlsServiceHandlerExecClient: ServiceHandlerExecClient + HasBlsServiceHandlerAddr
{
    async fn bls_service_handler_exec(
        &self,
        msg: &ExecuteMsg,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = HasBlsServiceHandlerAddr::addr(self);
        self.contract_exec(&contract_addr, msg, funds).await
    }
}

impl <T> BlsServiceHandlerExecClient for T
where
    T: ServiceHandlerExecClient + HasBlsServiceHandlerAddr { }