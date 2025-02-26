use super::Handle;
use crate::address_book;
use clap::{Parser, Subcommand};
use inquire::Text;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

/// Top level CLI struct
#[derive(Parser)]
#[command(name = "gm")]
#[command(about = "CLI tool for managing accounts and transactions")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
    pub fn handle(&self) {
        if self.command.is_none() {
            crate::gm::gm();
        }

        Commands::handle_optn_inquire(&self.command, ());
    }
}

/// First subcommand
///
/// Accounts - `gm acc`
/// Transactions - `gm tx`
#[derive(Subcommand, Display, EnumIter)]
enum Commands {
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

impl_inquire_selection!(Commands);

impl Handle for Commands {
    fn handle(&self, _carry_on: ()) {
        match self {
            Commands::Account { action } => {
                AccountActions::handle_optn_inquire(action, ());
            }
            Commands::AddressBook { action } => {
                address_book::AddressBookActions::handle_optn_inquire(action, ())
            }
            Commands::Transaction { action } => {
                TransactionActions::handle_optn_inquire(action, ());
            }
            Commands::SignMessage { message } => {
                let message = if message.is_empty() {
                    Text::new("Enter the message to sign:")
                        .prompt()
                        .expect("must enter message to sign")
                } else {
                    message.clone()
                };

                crate::sign_message::sign_message(message);
            }
        }
    }
}

#[derive(Subcommand, Display, EnumIter)]
enum AccountActions {
    #[command(alias = "new")]
    Create,

    #[command(alias = "ls")]
    List,
}

impl_inquire_selection!(AccountActions);

impl Handle for AccountActions {
    fn handle(&self, _carry_on: ()) {
        match self {
            AccountActions::List => {
                println!("Listing all accounts...");
                crate::account::list_of_wallets();
            }
            AccountActions::Create => {
                println!("Creating a new account...");
                crate::account::create_privatekey_wallet();
            }
        }
    }
}

/// Transaction subcommands
///
/// List - `gm tx ls`
/// Create - `gm tx new`
#[derive(Subcommand, Display, EnumIter)]
enum TransactionActions {
    #[command(alias = "ls")]
    List,

    #[command(alias = "new")]
    Create,
}

impl_inquire_selection!(TransactionActions);

impl Handle for TransactionActions {
    fn handle(&self, _carry_on: ()) {
        match self {
            TransactionActions::List => {
                println!("Listing all transactions...");
                // Implement listing logic
            }
            TransactionActions::Create => {
                println!("Creating a new transaction...");
                // Implement transaction creation logic
            }
        }
    }
}
