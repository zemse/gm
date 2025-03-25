pub mod account;
pub mod address_book;
pub mod balances;
pub mod config;
pub mod setup;
pub mod sign_message;
pub mod transaction;

use crate::utils::{Handle, Inquire};

use account::AccountActions;
use clap::Subcommand;
use config::ConfigActions;
use inquire::Text;
use setup::{get_setup_menu, setup_inquire_and_handle};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use transaction::TransactionActions;

/// First subcommand
///
/// Accounts - `gm acc`
/// Transactions - `gm tx`
#[derive(Subcommand, Display, EnumIter)]
#[allow(clippy::large_enum_variant)]
pub enum Action {
    #[command(hide = true)]
    Setup,

    #[command(alias = "bal")]
    Assets,

    #[command(alias = "acc")]
    Account {
        #[command(subcommand)]
        action: Option<AccountActions>,
    },

    #[command(alias = "ab")]
    AddressBook {
        #[command(subcommand)]
        action: Option<address_book::AddressBookActions>,
    },

    #[command(alias = "tx")]
    Transaction {
        #[command(subcommand)]
        action: Option<TransactionActions>,
    },

    #[command(alias = "sm")]
    SignMessage { message: Option<String> },

    #[command(alias = "cfg")]
    Config {
        #[command(subcommand)]
        action: Option<config::ConfigActions>,
    },
}

impl Inquire for Action {
    fn inquire(_: &()) -> Option<Action> {
        let mut options: Vec<Action> = Action::iter().collect();

        let setup_menu = get_setup_menu();
        if setup_menu.is_empty() {
            options.remove(0);
        }

        inquire::Select::new("Choose subcommand:", options)
            .with_formatter(&|a| format!("{a}"))
            .prompt()
            .ok()
    }
}

impl Handle for Action {
    fn handle(&self, _carry_on: ()) {
        match self {
            // TODO Add a setup option which helps user to enter any pending API keys or other configurations
            Action::Setup => setup_inquire_and_handle(),
            Action::Assets => {
                balances::get_all_balances();
            }
            Action::Account { action } => {
                AccountActions::handle_optn_inquire(action, ());
            }
            Action::AddressBook { action } => {
                address_book::AddressBookActions::handle_optn_inquire(action, ())
            }
            Action::Transaction { action } => {
                TransactionActions::handle_optn_inquire(action, ());
            }
            Action::SignMessage { message } => {
                let message = if let Some(message) = message {
                    message.clone()
                } else {
                    Text::new("Enter the message to sign:")
                        .prompt()
                        .expect("must enter message to sign")
                };

                sign_message::sign_message(message);
            }
            Action::Config { action } => {
                ConfigActions::handle_optn_inquire(action, ());
            }
        }
    }
}
