use cosmwasm_std::Addr;

pub trait HasEcdsaServiceHandlerAddr {
    fn addr(&self) -> Addr;
}