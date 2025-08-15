use cosmwasm_std::Addr;

pub trait HasMockServiceManagerAddr {
    fn addr(&self) -> Addr;
}