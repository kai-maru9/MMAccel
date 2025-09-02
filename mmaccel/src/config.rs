// mmaccel/src/config.rs

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub backup_interval_minutes: u64,
    pub backup_dir: String,
    pub max_backups: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backup_interval_minutes: 5, // デフォルトは5分
            backup_dir: "Backup".to_string(),
            max_backups: 10,
        }
    }
}

pub fn load_config() -> Config {
    let config_path = PathBuf::from("mmd_backup_config.toml");
    if !config_path.exists() {
        let default_config = Config::default();
        let toml_string = toml::to_string_pretty(&default_config).unwrap();
        if fs::write(&config_path, toml_string).is_ok() {
            return default_config;
        }
    }

    fs::read_to_string(config_path)
        .ok()
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}
