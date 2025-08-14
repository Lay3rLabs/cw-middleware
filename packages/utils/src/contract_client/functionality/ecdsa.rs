use async_trait::async_trait;

use crate::prelude::{WavsExecClientExt, WavsQueryClientExt};

#[async_trait(?Send)]
pub trait WavsEcdsaQueryClientExt: WavsQueryClientExt {
    async fn ecdsa_query_stuff(&self) {
        todo!()
    }
}

#[async_trait(?Send)]
pub trait WavsEcdsaExecClientExt: WavsExecClientExt {
    async fn ecdsa_exec_stuff(&self) {
        todo!()
    }
}