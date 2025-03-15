use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub api_id: i32,
    pub api_hash: String,
    pub phone: String,
    pub session_file_path: PathBuf,
    pub password: String,
}

impl Default for Config {
    fn default() -> Self {
        log::info!("Created default config, please edit it.");
        Self {
            api_id: 0,
            api_hash: String::new(),
            phone: String::new(),
            session_file_path: "session.bin".into(),
            password: "password".into(),
        }
    }
}
