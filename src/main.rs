use clap::Parser;
use gm::cli::Cli;

fn main() {
    let cli = Cli::parse();
    cli.handle();
}
