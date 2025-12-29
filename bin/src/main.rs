use clap::Parser;
use gm_ratatui_extra::widgets::popup::PopupWidget;
use gm_tui::{
    pages::{deploy_popup::DeployPopup, shell::ShellPage, walletconnect::WalletConnectPage, Page},
    Focus, MainMenuItem,
};
use gm_utils::{disk_storage::DiskStorageInterface, network::Network};

mod cli;
use crate::cli::{Cli, Commands};

mod panic_hook;

#[tokio::main]
async fn main() -> gm_tui::Result<()> {
    panic_hook::set();

    let mut tui_app = gm_tui::App::new()?;
    let main_menu = &mut tui_app.main_menu;

    let args = Cli::parse();

    let mut pre_events = None;

    if let Some(cmd) = args.cmd {
        match cmd.resolve_wildcard() {
            Commands::WalletConnect { uri } => {
                let mut wc = WalletConnectPage::new()?;
                if let Some(uri) = uri {
                    wc.set_uri(&uri);
                }

                main_menu.set_focussed_item(MainMenuItem::WalletConnect);
                tui_app.set_page(Page::WalletConnect(wc));
            }

            Commands::Shell {
                expose_private_key: _,
                cmd,
            } => {
                if cmd.is_empty() {
                    println!("Please provide a command to run");

                    return Ok(());
                }
                let run_page = ShellPage::from_command(cmd);

                main_menu.set_focussed_item(MainMenuItem::Shell);
                tui_app.set_page(Page::Shell(run_page));
                tui_app.update_focus(Focus::Body);
                tui_app.hide_main_menu = true;
                pre_events = Some(vec![gm_tui::AppEvent::INPUT_KEY_ENTER]);
            }

            Commands::InviteCode { code } => {
                tui_app.invite_popup.set_invite_code(code);
                tui_app.invite_popup.open();
            }

            Commands::Deploy { path, network } => {
                let network = network.map(|n| Network::from_name(&n)).transpose()?;
                let account = gm_utils::config::Config::load()?.get_current_account()?;

                tui_app.deploy_popup = DeployPopup::from_artifact_path(
                    &path,
                    network,
                    account,
                    &tui_app.shared_state().networks,
                )?;
                tui_app.hide_main_menu = true;
            }

            Commands::Wildcard(_) => unreachable!(),
        }
    }

    tui_app.run(pre_events).await?;

    Ok(())
}
