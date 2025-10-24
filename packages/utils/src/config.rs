use std::{collections::BTreeMap, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_types_full::{ChainKey, CosmosChainConfig};

use crate::path::repo_wavs_home;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ChainConfigs {
    pub cosmos: BTreeMap<ChainKey, CosmosChainConfig>,
}

impl ChainConfigs {
    pub async fn load_from_wavs(wavs_home: Option<impl AsRef<Path>>) -> Result<Self> {
        #[derive(Deserialize)]
        struct ConfigFile {
            default: ConfigDefault,
        }

        #[derive(Deserialize)]
        struct ConfigDefault {
            chains: ChainConfigs,
        }

        let wavs_home = match wavs_home {
            Some(path) => path.as_ref().to_path_buf(),
            None => repo_wavs_home()
                .ok_or_else(|| anyhow::anyhow!("Failed to determine WAVS home directory"))?,
        };

        let contents = tokio::fs::read_to_string(wavs_home.join("wavs.toml")).await?;
        let config: ConfigFile = toml::from_str(&contents)?;

        Ok(config.default.chains)
    }
}
