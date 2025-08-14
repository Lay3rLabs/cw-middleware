use async_trait::async_trait;

use crate::prelude::{WavsExecClientExt, WavsQueryClientExt};

#[async_trait(?Send)]
pub trait WavsBlsQueryClientExt: WavsQueryClientExt {
    async fn bls_query_stuff(&self) {
        todo!()
    }
}

#[async_trait(?Send)]
pub trait WavsBlsExecClientExt: WavsExecClientExt {
    async fn bls_exec_stuff(&self) {
        todo!()
    }
}