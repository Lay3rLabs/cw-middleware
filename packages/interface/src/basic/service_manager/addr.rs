use cosmwasm_std::Addr;

pub trait HasServiceManagerAddr {
    fn addr(&self) -> Addr;
}

