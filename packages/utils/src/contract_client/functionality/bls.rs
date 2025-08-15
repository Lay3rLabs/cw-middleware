use async_trait::async_trait;
use cosmwasm_std::Coin;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::prelude::{
    WavsBasicExecClientExt, WavsBasicQueryClientExt, WavsExecClientExt, WavsQueryClientExt,
    WavsServiceHandlerAddrExt, WavsServiceManagerAddrExt,
};

#[async_trait(?Send)]
pub trait WavsBlsQueryClientExt: WavsQueryClientExt {
    async fn bls_service_handler_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &bls_api::service_handler::QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let addr = WavsServiceHandlerAddrExt::addr(self.service_handler());
        self.service_handler()
            .basic_contract_query(&addr, msg)
            .await
    }
    async fn bls_service_manager_query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &bls_api::service_manager::QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        let addr = WavsServiceManagerAddrExt::addr(self.service_manager());

        self.service_handler()
            .basic_contract_query(&addr, msg)
            .await
    }
}

#[async_trait(?Send)]
pub trait WavsBlsExecClientExt: WavsExecClientExt {
    async fn bls_service_handler_exec(
        &self,
        msg: &bls_api::service_handler::ExecuteMsg,
        funds: &[Coin],
    ) -> Result<<Self::ServiceHandler as WavsBasicExecClientExt>::TxResponse, cosmwasm_std::StdError>
    {
        let addr = WavsServiceHandlerAddrExt::addr(self.service_handler());
        self.service_handler()
            .basic_contract_exec(&addr, msg, funds)
            .await
    }

    async fn bls_service_manager_exec(
        &self,
        msg: &bls_api::service_manager::ExecuteMsg,
        funds: &[Coin],
    ) -> Result<<Self::ServiceManager as WavsBasicExecClientExt>::TxResponse, cosmwasm_std::StdError>
    {
        let addr = WavsServiceManagerAddrExt::addr(self.service_manager());
        self.service_manager()
            .basic_contract_exec(&addr, msg, funds)
            .await
    }
}
