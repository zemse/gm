use clap::{Parser, Subcommand};
use inquire::Select;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use super::Handle;

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
        Commands::handle_optn(&self.command);
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

    #[command(alias = "tx")]
    Transaction {
        #[command(subcommand)]
        action: Option<TransactionActions>,
    },
}

impl_inquire_selection!(Commands);

impl Handle for Commands {
    fn handle(&self) {
        match self {
            Commands::Account { action } => {
                AccountActions::handle_optn(action);
            }
            Commands::Transaction { action } => {
                TransactionActions::handle_optn(action);
            }
        }
    }
}

/// Account subcommands
///
/// List - `gm acc ls`
/// Create - `gm acc new`
#[derive(Subcommand, Display, EnumIter)]
enum AccountActions {
    #[command(alias = "ls")]
    List,

    #[command(alias = "new")]
    Create,
}

impl_inquire_selection!(AccountActions);

impl Handle for AccountActions {
    fn handle(&self) {
        match self {
            AccountActions::List => {
                println!("Listing all accounts...");
                // Implement listing logic
            }
            AccountActions::Create => {
                println!("Creating a new account...");
                // Implement account creation logic
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
    fn handle(&self) {
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
