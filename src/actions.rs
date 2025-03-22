pub mod account;
pub mod address_book;
pub mod balances;
pub mod config;
pub mod setup;
pub mod sign_message;
pub mod transaction;
pub mod send_message;

use crate::utils::{Handle, Inquire};

use account::AccountActions;
use clap::Subcommand;
use config::ConfigActions;
use inquire::Text;
use setup::{get_setup_menu, setup_inquire_and_handle};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use transaction::TransactionActions;
use alloy::primitives::Address;
use crate::actions::send_message::send_message;

/// First subcommand
///
/// Accounts - `gm acc`
/// Transactions - `gm tx`
#[derive(Subcommand, Display, EnumIter)]
pub enum Action {
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
    SignMessage {
        message: String,
    },

    #[command(alias = "send")]
    SendMessage {
        /// Recipient address
        to: String,

        /// Message to send
        msg: String,

        // Network to use
       // network: String,
        
    },


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
                let message = if message.is_empty() {
                    Text::new("Enter the message to sign:")
                        .prompt()
                        .expect("must enter message to sign")
                } else {
                    message.clone()
                };

                sign_message::sign_message(message);
            }
            
            Action::SendMessage { to, msg } => {
                let to_address: Address = to.parse().expect("Invalid Ethereum address");
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(send_message(to_address, msg.clone()));
            }

            Action::Config { action } => {
                ConfigActions::handle_optn_inquire(action, ());
            }

        }
    }
}
