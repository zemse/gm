use clap::{CommandFactory, Parser, Subcommand};
use console::style;
use walletconnect_sdk::utils::UriParameters;

#[derive(Parser, Debug)]
#[clap(version, subcommand_required = false, arg_required_else_help = false)]
#[command(name = "gm", bin_name = "gm", version)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Connect to browser dApps using WalletConnect v2 URI
    #[command(alias = "wc")]
    WalletConnect { uri: Option<String> },

    /// Execute programs with gm's JSON RPC signer URL available in env
    #[command(alias = "run")]
    Shell {
        #[arg(long)]
        expose_private_key: bool,
        #[arg(trailing_var_arg = true)]
        cmd: Vec<String>,
    },

    #[command(alias = "its", hide = true)]
    InviteCode { code: String },

    #[command(external_subcommand)]
    Wildcard(#[allow(dead_code)] Vec<String>),
}

impl Commands {
    pub fn resolve_wildcard(self) -> Self {
        if let Commands::Wildcard(cmd) = self {
            assert!(!cmd.is_empty());
            let first = &cmd[0];

            // 1. WalletConnect URI
            let result = if UriParameters::try_from(first.clone()).is_ok() {
                Commands::WalletConnect {
                    uri: Some(first.clone()),
                }
            } else {
                eprintln!(
                    "{} unrecognized subcommand '{}'\n",
                    style("error:").red(),
                    style(cmd.join(" ")).yellow(),
                );
                Cli::command().print_help().unwrap();
                std::process::exit(1);
            };

            println!("{:?}", result);

            result
        } else {
            self
        }
    }
}
