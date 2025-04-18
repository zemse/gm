use clap::Parser;
use figlet_rs::FIGfont;
use gm_lib::{actions::Action, disk::Config, network::NetworkStore, tui, utils::Handle};
use inquire::Confirm;

#[tokio::main]
async fn main() -> gm_lib::Result<()> {
    preload_hook();

    let cli = Cli::parse();

    if cli.is_empty() {
        tui::run().await?;
    } else {
        cli.handle();
    }

    Ok(())
}

/// Top level CLI struct
#[derive(Parser)]
#[command(name = "gm")]
#[command(about = "CLI tool for managing accounts and transactions")]
pub struct Cli {
    #[command(subcommand)]
    action: Option<Action>,
}

impl Cli {
    pub fn handle(&self) {
        if self.action.is_none() {
            gm_art();
            println!("Welcome to GM CLI tool!");

            println!("Current account: {:?}\n", Config::current_account());

            let result = Confirm::new("Open menu?")
                .with_default(true)
                .with_help_message("Press ESC if you want to quit")
                .prompt();

            if let Ok(true) = result {
                Action::handle_optn_inquire(&None, ());
            }
        } else {
            Action::handle_optn_inquire(&self.action, ());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.action.is_none()
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

fn preload_hook() {
    // TODO its better to do it when it is needed instead of always
    NetworkStore::sort_config();
}
