use cosmwasm_std::Addr;

pub trait HasEcdsaServiceManagerAddr {
    fn addr(&self) -> Addr;
}