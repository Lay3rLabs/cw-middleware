use layer_climb_core::events::CosmosTxEvents;

pub trait TxResponseExt {
    fn extract_events(&self) -> CosmosTxEvents<'_>;
}