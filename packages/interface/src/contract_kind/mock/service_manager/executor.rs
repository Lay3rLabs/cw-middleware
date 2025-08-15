use async_trait::async_trait;
use cosmwasm_std::Coin;
use layer_climb_address::AddrEvm;
use mock_api::service_manager::ExecuteMsg;
use crate::ServiceManagerExecClient;

use super::HasMockServiceManagerAddr;


#[async_trait(?Send)]
pub trait MockServiceManagerExecClient: ServiceManagerExecClient + HasMockServiceManagerAddr
{
    async fn mock_service_manager_exec(
        &self,
        msg: &ExecuteMsg,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = HasMockServiceManagerAddr::addr(self);
        self.contract_exec(&contract_addr, msg, funds).await
    }

    async fn set_signing_key(
        &self,
        operator_addr: AddrEvm,
        signing_key_addr: AddrEvm,
        weight: u64,
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError>
    {
        self.mock_service_manager_exec(
            &mock_api::service_manager::ExecuteMsg::SetSigningKey {
                operator: operator_addr,
                signing_key: signing_key_addr,
                weight: weight.into(),
            },
            &[],
        )
        .await
    }
}

impl <T> MockServiceManagerExecClient for T
where
    T: ServiceManagerExecClient + HasMockServiceManagerAddr { }