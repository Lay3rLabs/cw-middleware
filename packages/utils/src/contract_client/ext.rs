use super::functionality::{
    WavsServiceHandlerExecClientExt, WavsServiceHandlerQueryClientExt,
    WavsServiceManagerExecClientExt, WavsServiceManagerQueryClientExt,
    WavsTriggerExecClientExt, WavsTriggerQueryClientExt,
};
use async_trait::async_trait;
use cosmwasm_std::{Addr, Coin};
use layer_climb::events::CosmosTxEvents;
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
    type TxResponse: TxResponseExt; 

    async fn basic_contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError>;
}

pub trait TxResponseExt {
    fn extract_events(&self) -> CosmosTxEvents<'_>;
}

impl TxResponseExt for cw_multi_test::AppResponse {
    fn extract_events(&self) -> CosmosTxEvents<'_> {
        CosmosTxEvents::from(self.events.as_slice())
    }
}

impl TxResponseExt for layer_climb::proto::abci::TxResponse {
    fn extract_events(&self) -> CosmosTxEvents<'_> {
        CosmosTxEvents::from(self)
    }
}

pub trait WavsServiceHandlerAddrExt {
    fn addr(&self) -> Addr;
}

pub trait WavsServiceManagerAddrExt {
    fn addr(&self) -> Addr;
}

pub trait WavsTriggerAddrExt {
    fn addr(&self) -> Addr;
}

// Helper containers for when we have all the clients
pub trait WavsQueryClientExt {
    type ServiceHandler: WavsServiceHandlerQueryClientExt;
    type ServiceManager: WavsServiceManagerQueryClientExt;
    type Trigger: WavsTriggerQueryClientExt;

    fn service_handler(&self) -> &Self::ServiceHandler;
    fn service_manager(&self) -> &Self::ServiceManager;
    fn trigger(&self) -> &Self::Trigger;
}

pub trait WavsExecClientExt {
    type ServiceHandler: WavsServiceHandlerExecClientExt;
    type ServiceManager: WavsServiceManagerExecClientExt;
    type Trigger: WavsTriggerExecClientExt;

    fn service_handler(&self) -> &Self::ServiceHandler;
    fn service_manager(&self) -> &Self::ServiceManager;
    fn trigger(&self) -> &Self::Trigger;
}

// Convenience for common cases of a wrapper that _has_ a member with the trait
// this will allow auto-implementing all the traits that flow from it

pub trait HasWavsQueryClient {
    type QueryClient: WavsQueryClientExt;

    fn query_client(&self) -> &Self::QueryClient;
}


pub trait HasWavsExecClient {
    type ExecClient: WavsExecClientExt;

    fn exec_client(&self) -> &Self::ExecClient;
}

// Easier to just accept this one trait everywhere
// nothing to implement, it's covered by blanket implementations below
pub trait WavsClientExt: WavsQueryClientExt + WavsExecClientExt {
    fn service_handler_querier(&self) -> &<Self as WavsQueryClientExt>::ServiceHandler;
    fn service_manager_querier(&self) -> &<Self as WavsQueryClientExt>::ServiceManager;
    fn trigger_querier(&self) -> &<Self as WavsQueryClientExt>::Trigger;
    fn service_handler_exec(&self) -> &<Self as WavsExecClientExt>::ServiceHandler;
    fn service_manager_exec(&self) -> &<Self as WavsExecClientExt>::ServiceManager;
    fn trigger_exec(&self) -> &<Self as WavsExecClientExt>::Trigger;
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

impl<T> WavsTriggerQueryClientExt for T where
    T: WavsBasicQueryClientExt + WavsTriggerAddrExt
{
}
impl<T> WavsTriggerExecClientExt for T where
    T: WavsBasicExecClientExt + WavsTriggerQueryClientExt
{
}

impl<T> WavsClientExt for T
where
    T: WavsQueryClientExt + WavsExecClientExt,
{
    fn service_handler_querier(&self) -> &<Self as WavsQueryClientExt>::ServiceHandler {
        <Self as WavsQueryClientExt>::service_handler(self)
    }

    fn service_manager_querier(&self) -> &<Self as WavsQueryClientExt>::ServiceManager {
        <Self as WavsQueryClientExt>::service_manager(self)
    }

    fn trigger_querier(&self) -> &<Self as WavsQueryClientExt>::Trigger{
        <Self as WavsQueryClientExt>::trigger(self)
    }

    fn service_handler_exec(&self) -> &<Self as WavsExecClientExt>::ServiceHandler {
        <Self as WavsExecClientExt>::service_handler(self)
    }

    fn service_manager_exec(&self) -> &<Self as WavsExecClientExt>::ServiceManager {
        <Self as WavsExecClientExt>::service_manager(self)
    }

    fn trigger_exec(&self) -> &<Self as WavsExecClientExt>::Trigger {
        <Self as WavsExecClientExt>::trigger(self)
    }
}

impl <T> WavsQueryClientExt for T where
    T: HasWavsQueryClient
{
    type ServiceHandler = <<T as HasWavsQueryClient>::QueryClient as WavsQueryClientExt>::ServiceHandler;
    type ServiceManager = <<T as HasWavsQueryClient>::QueryClient as WavsQueryClientExt>::ServiceManager;
    type Trigger = <<T as HasWavsQueryClient>::QueryClient as WavsQueryClientExt>::Trigger;

    fn service_handler(&self) -> &Self::ServiceHandler {
        self.query_client().service_handler()
    }

    fn service_manager(&self) -> &Self::ServiceManager {
        self.query_client().service_manager()
    }

    fn trigger(&self) -> &Self::Trigger {
        self.query_client().trigger()
    }
}

impl <T> WavsExecClientExt for T where
    T: HasWavsExecClient
{
    type ServiceHandler = <<T as HasWavsExecClient>::ExecClient as WavsExecClientExt>::ServiceHandler;
    type ServiceManager = <<T as HasWavsExecClient>::ExecClient as WavsExecClientExt>::ServiceManager;
    type Trigger = <<T as HasWavsExecClient>::ExecClient as WavsExecClientExt>::Trigger;

    fn service_handler(&self) -> &Self::ServiceHandler {
        self.exec_client().service_handler()
    }

    fn service_manager(&self) -> &Self::ServiceManager {
        self.exec_client().service_manager()
    }

    fn trigger(&self) -> &Self::Trigger {
        self.exec_client().trigger()
    }
}
