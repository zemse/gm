use clap::Parser;
use gm_cli::cli::Cli;

fn main() {
    let cli = Cli::parse();
    cli.handle();
}
