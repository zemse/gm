use clap::Parser;
use gm_tui::pages::{
    main_menu::MainMenuItem, shell::ShellPage, walletconnect::WalletConnectPage, Page,
};

mod cli;
use crate::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> gm_tui::Result<()> {
    let mut tui_app = gm_tui::App::new()?;
    let main_menu = tui_app
        .current_page_mut()
        .and_then(|p| p.as_main_menu_mut())
        .expect("current page not main menu");

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
                tui_app.insert_page(Page::WalletConnect(wc));
            }

            Commands::Shell {
                expose_private_key: _,
                cmd,
            } => {
                let mut run_page = ShellPage::default();
                if !cmd.is_empty() {
                    let (input, cursor) = run_page.get_user_input_mut().expect("not in input mode");
                    *input = cmd.join(" ");
                    *cursor = input.len();
                    pre_events = Some(vec![gm_tui::Event::INPUT_KEY_ENTER]);
                }
                main_menu.set_focussed_item(MainMenuItem::Shell);
                tui_app.insert_page(Page::Shell(run_page));
            }

            Commands::InviteCode { code } => {
                tui_app.invite_popup.set_invite_code(code);
                tui_app.invite_popup.open();
            }

            Commands::Wildcard(_) => unreachable!(),
        }
    }

    tui_app.run(pre_events).await?;

    Ok(())
}
