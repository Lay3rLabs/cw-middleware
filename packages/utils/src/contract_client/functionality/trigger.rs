use async_trait::async_trait;
use crate::prelude::{WavsExecClientExt, WavsQueryClientExt};

#[async_trait(?Send)]
pub trait WavsTriggerQueryClientExt: WavsQueryClientExt {
    async fn trigger_query_stuff(&self) {
        todo!()
    }
}

#[async_trait(?Send)]
pub trait WavsTriggerExecClientExt: WavsExecClientExt {
    async fn trigger_exec_stuff(&self) {
        todo!()
    }
}