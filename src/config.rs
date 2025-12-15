use std::{path::PathBuf, sync::Arc};

use easy_config_store::ConfigStore;
use eyre::Result;
use log::{debug, info};
use serde::{Deserialize, Serialize};

pub type Config = Arc<ConfigStore<ConfigInner>>;
pub fn config(path: PathBuf) -> Result<Config> {
    let config = ConfigStore::<ConfigInner>::read(path, "settings".to_string())?;

    info!("config parsing successful");
    debug!(
        "loaded configuration:\n{}",
        toml::to_string_pretty(&*config)?
    );

    Ok(config.arc())
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConfigInner {
    pub api_id: i32,
    pub api_hash: String,
    pub phone: String,
    pub session_file_path: PathBuf,
    pub password: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Default for ConfigInner {
    fn default() -> Self {
        log::info!("Created default config, please edit it.");
        Self {
            api_id: 0,
            api_hash: String::new(),
            phone: String::new(),
            session_file_path: "session.bin".into(),
            password: "password".into(),
            server_host: "0.0.0.0".into(),
            server_port: 8080,
        }
    }
}
