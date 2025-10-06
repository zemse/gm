use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::disk_storage::{DiskStorageInterface, FileFormat};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Config {
    current_account: Option<Address>,
    testnet_mode: bool,
    #[serde(default)]
    developer_mode: bool,
    alchemy_api_key: Option<String>,
    #[serde(default)]
    theme_name: String,
    #[serde(default)]
    helios_enabled: bool,
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
            .ok_or(crate::Error::CurrentAccountNotSet)
    }

    pub fn set_current_account(address: Address) -> crate::Result<()> {
        let mut config = Config::load()?;
        config.current_account = Some(address);
        config.save()?;
        Ok(())
    }

    #[inline]
    pub fn get_testnet_mode(&self) -> bool {
        self.testnet_mode
    }

    #[inline]
    pub fn get_developer_mode(&self) -> bool {
        self.developer_mode
    }

    pub fn alchemy_api_key(use_default: bool) -> crate::Result<String> {
        Config::load()?.get_alchemy_api_key(use_default)
    }

    pub fn get_alchemy_api_key(&self, use_default: bool) -> crate::Result<String> {
        let config = Config::load()?;

        config
            .alchemy_api_key
            .filter(|s| !s.is_empty())
            .or_else(|| {
                if use_default {
                    // default key has very limited free quota and activity is monitored
                    Some("T0Fv_dXv5Kepb_KIa69-JR_JDxXdABxG".to_string())
                } else {
                    None
                }
            })
            .ok_or(crate::Error::AlchemyApiKeyNotSet)
    }

    pub fn set_alchemy_api_key(alchemy_api_key: String) -> crate::Result<()> {
        let mut config = Config::load()?;
        if alchemy_api_key.is_empty() {
            config.alchemy_api_key = None;
        } else {
            config.alchemy_api_key = Some(alchemy_api_key);
        }
        config.save()?;
        Ok(())
    }

    #[inline]
    pub fn get_theme_name(&self) -> &str {
        &self.theme_name
    }

    #[inline]
    pub fn get_helios_enabled(&self) -> bool {
        self.helios_enabled
    }

    pub fn set_values(
        &mut self,
        alchemy_api_key: Option<String>,
        testnet_mode: bool,
        developer_mode: bool,
        theme_name: String,
        helios_enabled: bool,
    ) -> crate::Result<()> {
        self.alchemy_api_key = alchemy_api_key;
        self.testnet_mode = testnet_mode;
        self.developer_mode = developer_mode;
        self.theme_name = theme_name;
        self.helios_enabled = helios_enabled;
        self.save()?;
        Ok(())
    }
}
