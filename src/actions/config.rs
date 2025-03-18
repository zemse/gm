use std::fmt::Display;

use clap::{command, Subcommand};
use inquire::{Select, Text};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    disk::{Config, DiskInterface},
    impl_inquire_selection,
    utils::Handle,
};

#[derive(Subcommand, EnumIter)]
pub enum ConfigActions {
    #[command(alias = "alchemy")]
    AlchemyApiKey { key: Option<String> },

    #[command(alias = "dm")]
    TestnetMode { enabled: Option<bool> },
}

impl Display for ConfigActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = Config::load();
        match self {
            ConfigActions::AlchemyApiKey { .. } => {
                write!(
                    f,
                    "Alchemy API key: {}",
                    match config.alchemy_api_key {
                        Some(key) => key,
                        None => "None".to_string(),
                    }
                )
            }
            ConfigActions::TestnetMode { .. } => {
                write!(
                    f,
                    "Testnet Mode: {}",
                    match config.testnet_mode {
                        true => "Activated",
                        false => "Not activated",
                    }
                )
            }
        }
    }
}

impl_inquire_selection!(ConfigActions, ());

impl Handle for ConfigActions {
    fn handle(&self, _carry_on: ()) {
        let mut config = Config::load();
        match self {
            ConfigActions::AlchemyApiKey { key } => {
                let key = key
                    .clone()
                    .or_else(|| {
                        Some(
                            Text::new("Enter new Alchemy API key:")
                                .prompt()
                                .expect("must input alchemy API key"),
                        )
                    })
                    .expect("must have an api key");
                if key.is_empty() {
                    config.alchemy_api_key = None;
                } else {
                    config.alchemy_api_key = Some(key);
                }
            }
            ConfigActions::TestnetMode { enabled } => {
                let enabled = enabled
                    .or_else(|| {
                        Some(
                            match Select::new("Enable testnet mode?", vec!["Yes", "No"])
                                .prompt()
                                .expect("must input testnet mode")
                            {
                                "Yes" => true,
                                "No" => false,
                                _ => panic!("Invalid input"),
                            },
                        )
                    })
                    .expect("must have a testnet mode");

                config.testnet_mode = enabled;
            }
        }
        config.save();
    }
}
