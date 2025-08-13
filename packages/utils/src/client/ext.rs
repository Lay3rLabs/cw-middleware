use super::functionality::{
    WavsServiceHandlerExecClientExt, WavsServiceHandlerQueryClientExt,
    WavsServiceManagerExecClientExt, WavsServiceManagerQueryClientExt,
};
use async_trait::async_trait;
use cosmwasm_std::{Addr, Coin};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

/// Only need to impl these very basic methods
#[async_trait(?Send)]
pub trait WavsBasicQueryClientExt {
    async fn basic_contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError>;
}

#[async_trait(?Send)]
pub trait WavsBasicExecClientExt: WavsBasicQueryClientExt {
    type TxResponse;

    async fn basic_contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError>;
}

pub trait WavsServiceHandlerAddrExt {
    fn addr(&self) -> Addr;
}

pub trait WavsServiceManagerAddrExt {
    fn addr(&self) -> Addr;
}

// Helper containers for when we have both clients
pub trait WavsQueryClientExt {
    type ServiceHandler: WavsServiceHandlerQueryClientExt;
    type ServiceManager: WavsServiceManagerQueryClientExt;

    fn service_handler(&self) -> &Self::ServiceHandler;
    fn service_manager(&self) -> &Self::ServiceManager;
}

pub trait WavsExecClientExt {
    type ServiceHandler: WavsServiceHandlerExecClientExt;
    type ServiceManager: WavsServiceManagerExecClientExt;

    fn service_handler(&self) -> &Self::ServiceHandler;
    fn service_manager(&self) -> &Self::ServiceManager;
}

// Easier to just accept this one trait everywhere
// nothing to implement, it's covered by blanket implementations below
pub trait WavsClientExt: WavsQueryClientExt + WavsExecClientExt {
    fn service_handler_querier(&self) -> &<Self as WavsQueryClientExt>::ServiceHandler;
    fn service_manager_querier(&self) -> &<Self as WavsQueryClientExt>::ServiceManager;
    fn service_handler_exec(&self) -> &<Self as WavsExecClientExt>::ServiceHandler;
    fn service_manager_exec(&self) -> &<Self as WavsExecClientExt>::ServiceManager;
}

// automatic blanket implementations FTW!
impl<T> WavsServiceHandlerQueryClientExt for T where
    T: WavsBasicQueryClientExt + WavsServiceHandlerAddrExt
{
}

impl<T> WavsServiceManagerQueryClientExt for T where
    T: WavsBasicQueryClientExt + WavsServiceManagerAddrExt
{
}

impl<T> WavsServiceHandlerExecClientExt for T where
    T: WavsBasicExecClientExt + WavsServiceHandlerQueryClientExt
{
}

impl<T> WavsServiceManagerExecClientExt for T where
    T: WavsBasicExecClientExt + WavsServiceManagerQueryClientExt
{
}

impl<T> WavsClientExt for T
where
    T: WavsQueryClientExt + WavsExecClientExt,
{
    fn service_handler_querier(&self) -> &<Self as WavsQueryClientExt>::ServiceHandler {
        <Self as WavsQueryClientExt>::service_handler(&self)
    }

    fn service_manager_querier(&self) -> &<Self as WavsQueryClientExt>::ServiceManager {
        <Self as WavsQueryClientExt>::service_manager(&self)
    }

    fn service_handler_exec(&self) -> &<Self as WavsExecClientExt>::ServiceHandler {
        <Self as WavsExecClientExt>::service_handler(&self)
    }

    fn service_manager_exec(&self) -> &<Self as WavsExecClientExt>::ServiceManager {
        <Self as WavsExecClientExt>::service_manager(&self)
    }
}
