use figlet_rs::FIGfont;
use gm_lib::{network::NetworkStore, tui};

#[tokio::main]
async fn main() -> gm_lib::Result<()> {
    preload_hook();

    let args: Vec<String> = std::env::args().skip(1).collect();

    tui::run(args).await?;

    Ok(())
}

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
