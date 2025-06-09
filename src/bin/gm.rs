use clap::Parser;
use figlet_rs::FIGfont;
use gm_lib::{network::NetworkStore, tui};

#[tokio::main]
async fn main() -> gm_lib::Result<()> {
    preload_hook();

    // let cli = Cli::parse();

    tui::run().await?;

    Ok(())
}

/// Top level CLI struct
#[derive(Parser)]
#[command(name = "gm")]
#[command(about = "CLI tool for managing accounts and transactions")]
pub struct Cli;

#[allow(dead_code)]
fn gm_art() {
    // Load the standard font
    let standard_font = FIGfont::standard().unwrap();

    // Convert text "GM" into ASCII art
    let figure = standard_font.convert("gm");

    // Print the result
    match figure {
        Some(art) => println!("{art}"),
        None => println!("Failed to generate ASCII text."),
    }
}

fn preload_hook() {
    // TODO its better to do it when it is needed instead of always
    NetworkStore::sort_config().expect("NetworkStore::sort_config() failed");
}
