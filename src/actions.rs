pub mod account;
pub mod address_book;
pub mod sign_message;
pub mod transaction;

use crate::{impl_inquire_selection, traits::Handle};

use account::AccountActions;
use clap::Subcommand;
use inquire::Text;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use transaction::TransactionActions;

/// First subcommand
///
/// Accounts - `gm acc`
/// Transactions - `gm tx`
#[derive(Subcommand, Display, EnumIter)]
pub enum Action {
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
    SignMessage { message: String },
}

impl_inquire_selection!(Action, ());

impl Handle for Action {
    fn handle(&self, _carry_on: ()) {
        match self {
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
        }
    }
}
