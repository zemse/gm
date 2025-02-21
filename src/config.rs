// we can store things like settings here
// as well as account addresses and their names
// account address book

use alloy::primitives::Address;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub current_account: Address,
    pub debug_mode: bool,
}

impl Config {
    /// Get the path to the config file
    fn config_path() -> PathBuf {
        let project_dirs =
            ProjectDirs::from("xyz", "zemse", "gm").expect("Could not determine config directory");
        project_dirs.config_dir().join("config.toml") // Store as TOML
    }

    /// Load settings from a file
    pub fn load() -> Self {
        let path = Self::config_path();

        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
            toml::from_str(&content).unwrap_or_else(|_| Config::default())
        } else {
            Config::default()
        }
    }

    /// Save settings to a file
    pub fn save(&self) {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok(); // Ensure config directory exists
        }

        let content = toml::to_string_pretty(self).expect("Failed to serialize config");
        fs::write(path, content).expect("Failed to write config file");
    }
}
