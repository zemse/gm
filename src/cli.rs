pub mod account;
pub mod address_book;
pub mod sign_message;
pub mod transaction;

use crate::{
    disk::{Config, DiskInterface},
    impl_inquire_selection,
    traits::Handle,
};

use account::AccountActions;
use clap::{Parser, Subcommand};
use figlet_rs::FIGfont;
use inquire::Text;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use transaction::TransactionActions;

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
            gm_art();
            println!("Welcome to GM CLI tool!");

            let config = Config::load();
            println!("Current account: {:?}\n", config.current_account);
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

impl_inquire_selection!(Commands, ());

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

                sign_message::sign_message(message);
            }
        }
    }
}

fn gm_art() {
    // Load the standard font
    let standard_font = FIGfont::standard().unwrap();

    // Convert text "GM" into ASCII art
    let figure = standard_font.convert("gm");

    // Print the result
    match figure {
        Some(art) => println!("{}", art),
        None => println!("Failed to generate ASCII text."),
    }
}
