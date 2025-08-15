use async_trait::async_trait;
use cosmwasm_std::Coin;
use wavs_types::contracts::cosmwasm::service_handler::{ServiceHandlerExecuteMessages, WavsEnvelope, WavsSignatureData};

use crate::{ExecClientExt, ServiceHandlerQueryClient};


#[async_trait(?Send)]
pub trait ServiceHandlerExecClient:
    ExecClientExt + ServiceHandlerQueryClient
{
    async fn service_handler_exec(
        &self,
        msg: &ServiceHandlerExecuteMessages,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = self.addr();
        self.contract_exec(&contract_addr, msg, funds).await
    }

    async fn handle_signed_envelope(
        &self,
        envelope: WavsEnvelope,
        signature_data: WavsSignatureData,
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let msg = ServiceHandlerExecuteMessages::WavsHandleSignedEnvelope {
            envelope,
            signature_data,
        };
        self.service_handler_exec(&msg, &[]).await
    }
}

impl <T> ServiceHandlerExecClient for T
where
    T: ExecClientExt + ServiceHandlerQueryClient { }