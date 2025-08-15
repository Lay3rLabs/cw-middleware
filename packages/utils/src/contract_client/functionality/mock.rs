use crate::prelude::{
    WavsBasicExecClientExt, WavsBasicQueryClientExt, WavsExecClientExt, WavsQueryClientExt,
    WavsServiceHandlerAddrExt, WavsServiceManagerAddrExt,
};
use async_trait::async_trait;
use cosmwasm_std::{Coin, Uint64};
use layer_climb::prelude::*;
use mock_api::service_handler::TriggerMessageResponse;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[async_trait(?Send)]
pub trait WavsMockQueryClientExt: WavsQueryClientExt {
    async fn mock_service_handler_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &mock_api::service_handler::QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let addr = WavsServiceHandlerAddrExt::addr(self.service_handler());
        self.service_handler()
            .basic_contract_query(&addr, msg)
            .await
    }
    async fn mock_service_manager_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &mock_api::service_manager::QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let addr = WavsServiceManagerAddrExt::addr(self.service_manager());

        self.service_handler()
            .basic_contract_query(&addr, msg)
            .await
    }

    async fn get_service_handler_trigger_message(
        &self,
        trigger_id: Uint64,
    ) -> Result<String, cosmwasm_std::StdError> {
        let resp: TriggerMessageResponse = self
            .mock_service_handler_query(&mock_api::service_handler::QueryMsg::TriggerMessage {
                trigger_id,
            })
            .await?;

        Ok(resp.message)
    }
}

#[async_trait(?Send)]
pub trait WavsMockExecClientExt: WavsExecClientExt {
    async fn mock_service_handler_exec(
        &self,
        msg: &mock_api::service_handler::ExecuteMsg,
        funds: &[Coin],
    ) -> Result<<Self::ServiceHandler as WavsBasicExecClientExt>::TxResponse, cosmwasm_std::StdError>
    {
        let addr = WavsServiceHandlerAddrExt::addr(self.service_handler());
        self.service_handler()
            .basic_contract_exec(&addr, msg, funds)
            .await
    }

    async fn mock_service_manager_exec(
        &self,
        msg: &mock_api::service_manager::ExecuteMsg,
        funds: &[Coin],
    ) -> Result<<Self::ServiceManager as WavsBasicExecClientExt>::TxResponse, cosmwasm_std::StdError>
    {
        let addr = WavsServiceManagerAddrExt::addr(self.service_manager());
        self.service_manager()
            .basic_contract_exec(&addr, msg, funds)
            .await
    }

    async fn mock_service_manager_set_signing_key(
        &self,
        operator_addr: AddrEvm,
        signing_key_addr: AddrEvm,
        weight: u64,
    ) -> Result<<Self::ServiceManager as WavsBasicExecClientExt>::TxResponse, cosmwasm_std::StdError>
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

    async fn mock_service_handler_set_trigger_message(
        &self,
        trigger_id: Uint64,
        message: impl ToString,
    ) -> Result<<Self::ServiceHandler as WavsBasicExecClientExt>::TxResponse, cosmwasm_std::StdError>
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
