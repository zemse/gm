use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::disk_storage::{DiskStorageInterface, FileFormat};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub current_account: Option<Address>,
    pub testnet_mode: bool,
    #[serde(default)]
    pub developer_mode: bool,
    pub alchemy_api_key: Option<String>,
    // TODO theme_name must be enum, which means either move themes into utils or this module in tui
    #[serde(default = "default_theme_name")]
    pub theme_name: String,
}

// TODO this is a temporary fix, need to have theme_name as an enum which requires a refactor
impl Default for Config {
    fn default() -> Self {
        Self {
            current_account: None,
            testnet_mode: false,
            developer_mode: false,
            alchemy_api_key: None,
            theme_name: default_theme_name(),
        }
    }
}

fn default_theme_name() -> String {
    "Monochrome".to_string()
}

impl DiskStorageInterface for Config {
    const FILE_NAME: &'static str = "config";
    const FORMAT: FileFormat = FileFormat::TOML;
}

impl Config {
    pub fn current_account() -> crate::Result<Address> {
        Config::load()?.get_current_account()
    }

    pub fn get_current_account(&self) -> crate::Result<Address> {
        self.current_account
            .ok_or_else(|| crate::Error::CurrentAccountNotSet)
    }

    pub fn set_current_account(address: Address) -> crate::Result<()> {
        let mut config = Config::load()?;
        config.current_account = Some(address);
        config.save()?;
        Ok(())
    }

    pub fn alchemy_api_key() -> crate::Result<String> {
        Config::load()?
            .alchemy_api_key
            .ok_or(crate::Error::AlchemyApiKeyNotSet)
    }

    pub fn set_alchemy_api_key(alchemy_api_key: String) -> crate::Result<()> {
        let mut config = Config::load()?;
        config.alchemy_api_key = Some(alchemy_api_key);
        config.save()?;
        Ok(())
    }
}
