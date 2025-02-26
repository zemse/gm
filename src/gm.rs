use crate::disk::{Config, DiskInterface};

use figlet_rs::FIGfont;

pub fn gm() {
    gm_art();
    println!("Welcome to GM CLI tool!");

    let config = Config::load();
    println!("Current account: {:?}\n", config.current_account);
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
