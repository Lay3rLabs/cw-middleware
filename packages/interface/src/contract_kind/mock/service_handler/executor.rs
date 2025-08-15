use async_trait::async_trait;
use cosmwasm_std::{Coin, Uint64};
use mock_api::service_handler::ExecuteMsg;
use super::HasMockServiceHandlerAddr;
use crate::ServiceHandlerExecClient;


#[async_trait(?Send)]
pub trait MockServiceHandlerExecClient: ServiceHandlerExecClient + HasMockServiceHandlerAddr
{
    async fn mock_service_handler_exec(
        &self,
        msg: &ExecuteMsg,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = HasMockServiceHandlerAddr::addr(self);
        self.contract_exec(&contract_addr, msg, funds).await
    }

    async fn set_trigger_message(
        &self,
        trigger_id: Uint64,
        message: impl ToString,
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError>
    {
        self.mock_service_handler_exec(
            &mock_api::service_handler::ExecuteMsg::SetTriggerMessage {
                trigger_id,
                message: message.to_string(),
            },
            &[],
        )
        .await
    }
}

impl <T> MockServiceHandlerExecClient for T
where
    T: ServiceHandlerExecClient + HasMockServiceHandlerAddr { }