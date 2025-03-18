use std::fmt::Display;

use inquire::{Select, Text};

use crate::disk::{Config, DiskInterface};

use super::account::create_privatekey_wallet;

pub enum SetupActions {
    CreateWallet,
    AlchemyApiKey,
}

impl Display for SetupActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetupActions::CreateWallet => {
                write!(f, "Create wallet or import existing wallet in GM")
            }
            SetupActions::AlchemyApiKey => {
                write!(f, "Set ALCHEMY_API_KEY for smooth ethereum access")
            }
        }
    }
}

// If this is empty we don't need to setup
pub fn get_setup_menu() -> Vec<SetupActions> {
    let mut setup_menu = vec![];
    let config = Config::load();
    if config.current_account.is_none() {
        setup_menu.push(SetupActions::CreateWallet);
    }
    if config.alchemy_api_key.is_none() {
        setup_menu.push(SetupActions::AlchemyApiKey)
    }
    setup_menu
}

pub fn setup_inquire_and_handle() {
    let result = Select::new("Choose:", get_setup_menu())
        .prompt()
        .expect("Must select something");

    match result {
        SetupActions::CreateWallet => {
            let address = create_privatekey_wallet();
            Config::set_current_account(address);
        }
        SetupActions::AlchemyApiKey => {
            let key = Text::new("Enter new Alchemy API key:")
                .prompt()
                .expect("must input alchemy API key");
            if key.is_empty() {
                panic!("Alchemy API key not be empty");
            }
            Config::set_alchemy_api_key(key);
        }
    };
}
