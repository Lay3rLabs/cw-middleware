use cosmwasm_std::Addr;

pub trait HasSimpleTriggerAddr {
    fn addr(&self) -> Addr;
}