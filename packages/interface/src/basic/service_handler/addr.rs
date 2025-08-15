use cosmwasm_std::Addr;

pub trait HasServiceHandlerAddr {
    fn addr(&self) -> Addr;
}

