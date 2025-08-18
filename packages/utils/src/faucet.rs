use anyhow::Result;
use layer_climb::prelude::*;
use serde::Serialize;

pub async fn tap(addr: &Address, denom: &str) -> Result<()> {
    #[derive(Serialize)]
    pub struct TapRequest<'a> {
        pub address: String,
        pub denom: &'a str,
    }

    reqwest::Client::new()
        .post("http://localhost:8001/credit")
        .json(&TapRequest {
            address: addr.to_string(),
            denom,
        })
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
