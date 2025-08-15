use cosmwasm_std::Addr;

pub trait HasBlsServiceHandlerAddr {
    fn addr(&self) -> Addr;
}
