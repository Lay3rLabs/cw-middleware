use async_trait::async_trait;
use cosmwasm_std::{Addr, Coin};
use layer_climb::{pool::SigningClientPool, prelude::*, proto::abci::TxResponse};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::{
    client::ext::{
        WavsBasicExecClientExt, WavsBasicQueryClientExt, WavsServiceHandlerAddrExt,
        WavsServiceManagerAddrExt,
    },
    prelude::{WavsExecClientExt, WavsQueryClientExt},
};

pub struct WavsQueryClient {
    service_handler: WavsServiceHandlerQueryClient,
    service_manager: WavsServiceManagerQueryClient,
}

impl WavsQueryClientExt for WavsQueryClient {
    type ServiceHandler = WavsServiceHandlerQueryClient;
    type ServiceManager = WavsServiceManagerQueryClient;

    fn service_handler(&self) -> &Self::ServiceHandler {
        &self.service_handler
    }

    fn service_manager(&self) -> &Self::ServiceManager {
        &self.service_manager
    }
}

pub struct WavsSigningPoolClient {
    #[allow(dead_code)]
    querier: QueryClient,
    #[allow(dead_code)]
    pool: SigningClientPool,
    service_handler_querier: WavsServiceHandlerQueryClient,
    service_manager_querier: WavsServiceManagerQueryClient,
    service_handler_exec: WavsServiceHandlerSigningPoolClient,
    service_manager_exec: WavsServiceManagerSigningPoolClient,
}

impl WavsQueryClientExt for WavsSigningPoolClient {
    type ServiceHandler = WavsServiceHandlerQueryClient;
    type ServiceManager = WavsServiceManagerQueryClient;

    fn service_handler(&self) -> &Self::ServiceHandler {
        &self.service_handler_querier
    }

    fn service_manager(&self) -> &Self::ServiceManager {
        &self.service_manager_querier
    }
}
impl WavsExecClientExt for WavsSigningPoolClient {
    type ServiceHandler = WavsServiceHandlerSigningPoolClient;
    type ServiceManager = WavsServiceManagerSigningPoolClient;

    fn service_handler(&self) -> &Self::ServiceHandler {
        &self.service_handler_exec
    }

    fn service_manager(&self) -> &Self::ServiceManager {
        &self.service_manager_exec
    }
}

pub struct WavsSigningClient {
    service_handler_querier: WavsServiceHandlerQueryClient,
    service_manager_querier: WavsServiceManagerQueryClient,
    service_handler_exec: WavsServiceHandlerSigningClient,
    service_manager_exec: WavsServiceManagerSigningClient,
}

impl WavsQueryClientExt for WavsSigningClient {
    type ServiceHandler = WavsServiceHandlerQueryClient;
    type ServiceManager = WavsServiceManagerQueryClient;

    fn service_handler(&self) -> &Self::ServiceHandler {
        &self.service_handler_querier
    }

    fn service_manager(&self) -> &Self::ServiceManager {
        &self.service_manager_querier
    }
}

impl WavsExecClientExt for WavsSigningClient {
    type ServiceHandler = WavsServiceHandlerSigningClient;
    type ServiceManager = WavsServiceManagerSigningClient;

    fn service_handler(&self) -> &Self::ServiceHandler {
        &self.service_handler_exec
    }

    fn service_manager(&self) -> &Self::ServiceManager {
        &self.service_manager_exec
    }
}

pub struct WavsServiceManagerQueryClient {
    querier: QueryClient,
    addr: Addr,
}
pub struct WavsServiceManagerSigningPoolClient {
    pool: SigningClientPool,
    addr: Addr,
}

pub struct WavsServiceManagerSigningClient {
    client: SigningClient,
    addr: Addr,
}

pub struct WavsServiceHandlerQueryClient {
    querier: QueryClient,
    addr: Addr,
}
pub struct WavsServiceHandlerSigningPoolClient {
    pool: SigningClientPool,
    addr: Addr,
}
pub struct WavsServiceHandlerSigningClient {
    client: SigningClient,
    addr: Addr,
}

impl WavsQueryClient {
    pub fn new(
        querier: QueryClient,
        service_handler_addr: &Address,
        service_manager_addr: &Address,
    ) -> Self {
        Self {
            service_handler: WavsServiceHandlerQueryClient::new(
                querier.clone(),
                service_handler_addr,
            ),
            service_manager: WavsServiceManagerQueryClient::new(querier, service_manager_addr),
        }
    }
}
impl WavsSigningPoolClient {
    pub fn new(
        querier: QueryClient,
        pool: SigningClientPool,
        service_handler_addr: &Address,
        service_manager_addr: &Address,
    ) -> Self {
        Self {
            querier: querier.clone(),
            pool: pool.clone(),
            service_handler_querier: WavsServiceHandlerQueryClient::new(
                querier.clone(),
                service_handler_addr,
            ),
            service_manager_querier: WavsServiceManagerQueryClient::new(
                querier,
                service_manager_addr,
            ),
            service_handler_exec: WavsServiceHandlerSigningPoolClient::new(
                pool.clone(),
                service_handler_addr,
            ),
            service_manager_exec: WavsServiceManagerSigningPoolClient::new(
                pool,
                service_manager_addr,
            ),
        }
    }
}

impl WavsSigningClient {
    pub fn new(
        client: SigningClient,
        service_handler_addr: &Address,
        service_manager_addr: &Address,
    ) -> Self {
        let querier = client.querier.clone();

        Self {
            service_handler_querier: WavsServiceHandlerQueryClient::new(
                querier.clone(),
                service_handler_addr,
            ),
            service_manager_querier: WavsServiceManagerQueryClient::new(
                querier,
                service_manager_addr,
            ),
            service_handler_exec: WavsServiceHandlerSigningClient::new(
                client.clone(),
                service_handler_addr,
            ),
            service_manager_exec: WavsServiceManagerSigningClient::new(
                client,
                service_manager_addr,
            ),
        }
    }
}

impl WavsServiceManagerQueryClient {
    pub fn new(querier: QueryClient, addr: &Address) -> Self {
        Self {
            querier,
            addr: Addr::unchecked(addr.to_string()),
        }
    }
}

impl WavsServiceManagerSigningPoolClient {
    pub fn new(pool: SigningClientPool, addr: &Address) -> Self {
        Self {
            pool,
            addr: Addr::unchecked(addr.to_string()),
        }
    }
}

impl WavsServiceManagerSigningClient {
    pub fn new(client: SigningClient, addr: &Address) -> Self {
        Self {
            client,
            addr: Addr::unchecked(addr.to_string()),
        }
    }
}

impl WavsServiceHandlerQueryClient {
    pub fn new(querier: QueryClient, addr: &Address) -> Self {
        Self {
            querier,
            addr: Addr::unchecked(addr.to_string()),
        }
    }
}

impl WavsServiceHandlerSigningPoolClient {
    pub fn new(pool: SigningClientPool, addr: &Address) -> Self {
        Self {
            pool,
            addr: Addr::unchecked(addr.to_string()),
        }
    }
}

impl WavsServiceHandlerSigningClient {
    pub fn new(client: SigningClient, addr: &Address) -> Self {
        Self {
            client,
            addr: Addr::unchecked(addr.to_string()),
        }
    }
}

#[async_trait(?Send)]
impl WavsBasicQueryClientExt for WavsServiceManagerQueryClient {
    async fn basic_contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        querier_contract_query(&self.querier, address, msg).await
    }
}

#[async_trait(?Send)]
impl WavsBasicQueryClientExt for WavsServiceManagerSigningPoolClient {
    async fn basic_contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        pool_contract_query(&self.pool, address, msg).await
    }
}

#[async_trait(?Send)]
impl WavsBasicQueryClientExt for WavsServiceManagerSigningClient {
    async fn basic_contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        client_contract_query(&self.client, address, msg).await
    }
}

#[async_trait(?Send)]
impl WavsBasicQueryClientExt for WavsServiceHandlerQueryClient {
    async fn basic_contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        querier_contract_query(&self.querier, address, msg).await
    }
}

#[async_trait(?Send)]
impl WavsBasicQueryClientExt for WavsServiceHandlerSigningPoolClient {
    async fn basic_contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        pool_contract_query(&self.pool, address, msg).await
    }
}

#[async_trait(?Send)]
impl WavsBasicQueryClientExt for WavsServiceHandlerSigningClient {
    async fn basic_contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        client_contract_query(&self.client, address, msg).await
    }
}

#[async_trait(?Send)]
impl WavsBasicExecClientExt for WavsServiceHandlerSigningPoolClient {
    type TxResponse = TxResponse;

    async fn basic_contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        pool_contract_exec(&self.pool, address, msg, funds).await
    }
}

#[async_trait(?Send)]
impl WavsBasicExecClientExt for WavsServiceHandlerSigningClient {
    type TxResponse = TxResponse;

    async fn basic_contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        client_contract_exec(&self.client, address, msg, funds).await
    }
}

#[async_trait(?Send)]
impl WavsBasicExecClientExt for WavsServiceManagerSigningPoolClient {
    type TxResponse = TxResponse;

    async fn basic_contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        pool_contract_exec(&self.pool, address, msg, funds).await
    }
}

#[async_trait(?Send)]
impl WavsBasicExecClientExt for WavsServiceManagerSigningClient {
    type TxResponse = TxResponse;

    async fn basic_contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        client_contract_exec(&self.client, address, msg, funds).await
    }
}

impl WavsServiceManagerAddrExt for WavsServiceManagerQueryClient {
    fn addr(&self) -> Addr {
        self.addr.clone()
    }
}

impl WavsServiceManagerAddrExt for WavsServiceManagerSigningClient {
    fn addr(&self) -> Addr {
        self.addr.clone()
    }
}

impl WavsServiceManagerAddrExt for WavsServiceManagerSigningPoolClient {
    fn addr(&self) -> Addr {
        self.addr.clone()
    }
}

impl WavsServiceHandlerAddrExt for WavsServiceHandlerQueryClient {
    fn addr(&self) -> Addr {
        self.addr.clone()
    }
}

impl WavsServiceHandlerAddrExt for WavsServiceHandlerSigningClient {
    fn addr(&self) -> Addr {
        self.addr.clone()
    }
}

impl WavsServiceHandlerAddrExt for WavsServiceHandlerSigningPoolClient {
    fn addr(&self) -> Addr {
        self.addr.clone()
    }
}

async fn client_contract_query<
    RESP: DeserializeOwned + Send + Sync + Debug,
    MSG: Serialize + Debug,
>(
    client: &SigningClient,
    address: &Addr,
    msg: &MSG,
) -> Result<RESP, cosmwasm_std::StdError> {
    let resp = querier_contract_query(&client.querier, address, msg).await?;

    Ok(resp)
}

async fn pool_contract_query<
    RESP: DeserializeOwned + Send + Sync + Debug,
    MSG: Serialize + Debug,
>(
    pool: &SigningClientPool,
    address: &Addr,
    msg: &MSG,
) -> Result<RESP, cosmwasm_std::StdError> {
    let client = pool
        .get()
        .await
        .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
    let resp = querier_contract_query(&client.querier, address, msg).await?;

    Ok(resp)
}

async fn querier_contract_query<
    RESP: DeserializeOwned + Send + Sync + Debug,
    MSG: Serialize + Debug,
>(
    querier: &QueryClient,
    address: &Addr,
    msg: &MSG,
) -> Result<RESP, cosmwasm_std::StdError> {
    let address = querier
        .chain_config
        .parse_address(address.as_str())
        .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;

    let resp = querier
        .contract_smart(&address, msg)
        .await
        .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;

    Ok(resp)
}

async fn pool_contract_exec<MSG: Serialize + std::fmt::Debug>(
    pool: &SigningClientPool,
    address: &Addr,
    msg: &MSG,
    funds: &[Coin],
) -> Result<TxResponse, cosmwasm_std::StdError> {
    let client = pool
        .get()
        .await
        .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
    let address = client
        .querier
        .chain_config
        .parse_address(address.as_str())
        .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
    let funds = funds
        .iter()
        .map(|c| layer_climb::prelude::Coin {
            denom: c.denom.clone(),
            amount: c.amount.to_string(),
        })
        .collect::<Vec<_>>();

    let resp = client
        .contract_execute(&address, msg, funds, None)
        .await
        .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;

    Ok(resp)
}

async fn client_contract_exec<MSG: Serialize + std::fmt::Debug>(
    client: &SigningClient,
    address: &Addr,
    msg: &MSG,
    funds: &[Coin],
) -> Result<TxResponse, cosmwasm_std::StdError> {
    let address = client
        .querier
        .chain_config
        .parse_address(address.as_str())
        .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
    let funds = funds
        .iter()
        .map(|c| layer_climb::prelude::Coin {
            denom: c.denom.clone(),
            amount: c.amount.to_string(),
        })
        .collect::<Vec<_>>();

    let resp = client
        .contract_execute(&address, msg, funds, None)
        .await
        .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;

    Ok(resp)
}
