pub mod account;
pub mod address_book;
pub mod balances;
pub mod config;
pub mod send_message;
pub mod setup;
pub mod sign_message;
pub mod transaction;
pub mod receive_payment;

use crate::utils::{Handle, Inquire};

use crate::disk::{AddressBook, DiskInterface};
use crate::network::{Network, NetworkStore};
use account::AccountActions;
use clap::Subcommand;
use config::ConfigActions;
use inquire::{Select, Text};
use setup::{get_setup_menu, setup_inquire_and_handle};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use transaction::TransactionActions;

/// First subcommand
///
/// Accounts - `gm acc`
/// Transactions - `gm tx`
#[derive(Subcommand, Display, Debug, EnumIter)]
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

    #[command(alias = "send")]
    SendMessage {
        /// Recipient address
        to: Option<String>,

        /// Message to send
        msg: Option<String>,

        // Network to use
        network: Option<Network>,
    },

     #[command(alias = "recv")]
    /// Receive payment
       ReceivePayment,

    #[command(alias = "cfg")]
    Config {
        #[command(subcommand)]
        action: Option<config::ConfigActions>,
    },
}

impl Action {
    pub fn get_menu() -> Vec<Action> {
        let mut options: Vec<Action> = Action::iter().collect();

        let setup_menu = get_setup_menu();
        if setup_menu.is_empty() {
            options.remove(0);
        }

        options
    }

    pub fn get_menu_str() -> Vec<String> {
        Action::get_menu()
            .into_iter()
            .map(|action| format!("{action}"))
            .collect()
    }
}

impl Inquire for Action {
    fn inquire(_: &()) -> Option<Action> {
        let options: Vec<Action> = Action::get_menu();

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

            Action::SendMessage { to, msg, network } => {
                let to = match to {
                    Some(addr) if !addr.is_empty() => addr.clone(),
                    _ => {
                        let choice = Select::new(
                            "Select recipient method:",
                            vec!["Enter manually", "Choose from address book"],
                        )
                        .prompt()
                        .expect("❌ Must select a method");
                        if choice == "Enter manually" {
                            Text::new("Enter recipient address:")
                                .prompt()
                                .expect("❌ Must enter recipient address")
                        } else {
                            let address_book = AddressBook::load();
                            let addresses = address_book.list().to_vec();

                            if addresses.is_empty() {
                                println!("⚠️ Address book is empty, please enter manually.");
                                Text::new("Enter recipient address:")
                                    .prompt()
                                    .expect("❌ Must enter recipient address")
                            } else {
                                let selected = Select::new("Select recipient:", addresses)
                                    .prompt()
                                    .expect("❌ Must select an address");
                                selected.address.to_string()
                            }
                        }
                    }
                };

                let msg = match msg {
                    Some(m) if !m.is_empty() => m.clone(),
                    _ => Text::new("Enter message:")
                        .prompt()
                        .expect("❌ Must enter a message"),
                };

                let network = network.clone().or_else(|| {
                    let networks = NetworkStore::load().networks;
                    Select::new("Select a network:", networks).prompt().ok()
                });

                send_message::handle_send_message(to, msg, network);
            }

            Action::ReceivePayment => {

                let req = 
                crate::actions::receive_payment::PaymentRequest::from_user_input();
                println!("🔗 {}", req.generate_link());               
            }


            Action::Config { action } => {
                ConfigActions::handle_optn_inquire(action, ());
            }
        }
    }
}
