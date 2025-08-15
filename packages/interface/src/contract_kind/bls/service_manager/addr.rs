use cosmwasm_std::Addr;

pub trait HasBlsServiceManagerAddr {
    fn addr(&self) -> Addr;
}