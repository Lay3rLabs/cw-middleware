use cosmwasm_std::Addr;

pub trait HasMockServiceHandlerAddr {
    fn addr(&self) -> Addr;
}