use cosmwasm_std::Addr;
use layer_climb::prelude::*;

#[derive(Clone)]
pub struct TestApp {
    pub pool: SigningClientPool,
    pub querier: QueryClient,
    service_handler_address: Addr,
    service_manager_addr: Addr,
    trigger_addr: Addr,
}

impl TestApp {
    pub fn new(
        pool: SigningClientPool,
        querier: QueryClient,
        service_handler_address: Addr,
        service_manager_addr: Addr,
        trigger_addr: Addr,
    ) -> Self {
        Self {
            pool,
            querier,
            service_handler_address,
            service_manager_addr,
            trigger_addr,
        }
    }
}
