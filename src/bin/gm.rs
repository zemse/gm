use clap::Parser;
use figlet_rs::FIGfont;
use gm_cli::{
    actions::Action,
    disk::{Config, DiskInterface},
    traits::Handle,
};

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

            let config = Config::load();
            println!("Current account: {:?}\n", config.current_account);
        }

        Action::handle_optn_inquire(&self.action, ());
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

fn main() {
    let cli = Cli::parse();
    cli.handle();
}
