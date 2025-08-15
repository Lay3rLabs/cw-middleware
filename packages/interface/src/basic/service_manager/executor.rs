use async_trait::async_trait;
use cosmwasm_std::Coin;
use wavs_types::contracts::cosmwasm::service_manager::ServiceManagerExecuteMessages;

use crate::{ExecClientExt, ServiceManagerQueryClient};

#[async_trait(?Send)]
pub trait ServiceManagerExecClient:
    ExecClientExt + ServiceManagerQueryClient
{
    async fn service_manager_exec(
        &self,
        msg: &ServiceManagerExecuteMessages,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = self.addr();
        self.contract_exec(&contract_addr, msg, funds).await
    }

    async fn set_service_uri(
        &self,
        uri: String,
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let msg = ServiceManagerExecuteMessages::WavsSetServiceUri { service_uri: uri };
        self.service_manager_exec(&msg, &[]).await
    }
}

impl <T> ServiceManagerExecClient for T
where
    T: ExecClientExt + ServiceManagerQueryClient { }