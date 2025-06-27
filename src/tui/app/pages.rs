use std::sync::{atomic::AtomicBool, mpsc, Arc};

use account::AccountPage;
use account_create::AccountCreatePage;
use account_import::AccountImportPage;
use address_book::AddressBookPage;
use address_book_create::AddressBookCreatePage;
use address_book_display::AddressBookDisplayPage;
use asset_transfer::AssetTransferPage;
use assets::AssetsPage;
use config::ConfigPage;
use dev_key_capture::DevKeyCapturePage;
use main_menu::{MainMenuItem, MainMenuPage};
use send_message::SendMessagePage;
use setup::SetupPage;
use sign_message::SignMessagePage;
use text::TextPage;
use trade::TradePage;
use walletconnect::WalletConnectPage;

use crate::tui::{
    events::Event,
    traits::{Component, HandleResult},
};

use super::SharedState;

pub mod account;
pub mod account_create;
pub mod account_import;
pub mod address_book;
pub mod address_book_create;
pub mod address_book_display;
pub mod asset_transfer;
pub mod assets;
pub mod config;
pub mod dev_key_capture;
pub mod main_menu;
pub mod send_message;
pub mod setup;
pub mod sign_message;
pub mod text;
pub mod trade;
pub mod walletconnect;

#[allow(clippy::large_enum_variant)]
pub enum Page {
    MainMenu(MainMenuPage),
    Setup(SetupPage),

    Account(AccountPage),
    AccountCreate(AccountCreatePage),
    AccountImport(AccountImportPage),

    AddressBook(AddressBookPage),
    AddressBookCreate(AddressBookCreatePage),
    AddressBookDisplay(AddressBookDisplayPage),

    Assets(AssetsPage),
    AssetTransfer(AssetTransferPage),

    Config(ConfigPage),
    SendMessage(SendMessagePage),
    SignMessage(SignMessagePage),
    // Transaction(TransactionPage),
    WalletConnect(WalletConnectPage),

    Trade(TradePage),

    Text(TextPage),
    DevKeyCapture(DevKeyCapturePage),
}

impl Page {
    pub fn is_full_screen(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match self {
            Page::AddressBookDisplay(_) => true,
            Page::Trade(_) => true,
            Page::SendMessage(_) => true,
            Page::SignMessage(_) => true,
            _ => false,
        }
    }

    pub fn is_main_menu(&self) -> bool {
        matches!(self, Page::MainMenu(_))
    }

    pub fn main_menu_focused_item(&self) -> Option<&MainMenuItem> {
        match self {
            Page::MainMenu(page) => Some(page.get_focussed_item()),
            _ => None,
        }
    }
}

impl Component for Page {
    fn set_focus(&mut self, focus: bool) {
        match self {
            Page::MainMenu(page) => page.set_focus(focus),
            Page::Setup(page) => page.set_focus(focus),

            Page::AddressBook(page) => page.set_focus(focus),
            Page::AddressBookCreate(page) => page.set_focus(focus),
            Page::AddressBookDisplay(page) => page.set_focus(focus),

            Page::Account(page) => page.set_focus(focus),
            Page::AccountCreate(page) => page.set_focus(focus),
            Page::AccountImport(page) => page.set_focus(focus),

            Page::Assets(page) => page.set_focus(focus),
            Page::AssetTransfer(page) => page.set_focus(focus),

            Page::Config(page) => page.set_focus(focus),
            Page::SendMessage(page) => page.set_focus(focus),
            Page::SignMessage(page) => page.set_focus(focus),
            // Page::Transaction(page) => page.set_focus(focus),
            Page::WalletConnect(page) => page.set_focus(focus),

            Page::Trade(page) => page.set_focus(focus),

            Page::Text(page) => page.set_focus(focus),
            Page::DevKeyCapture(page) => page.set_focus(focus),
        }
    }

    async fn exit_threads(&mut self) {
        match self {
            Page::MainMenu(page) => page.exit_threads().await,
            Page::Setup(page) => page.exit_threads().await,

            Page::AddressBook(page) => page.exit_threads().await,
            Page::AddressBookCreate(page) => page.exit_threads().await,
            Page::AddressBookDisplay(page) => page.exit_threads().await,

            Page::Account(page) => page.exit_threads().await,
            Page::AccountCreate(page) => page.exit_threads().await,
            Page::AccountImport(page) => page.exit_threads().await,

            Page::Assets(page) => page.exit_threads().await,
            Page::AssetTransfer(page) => page.exit_threads().await,

            Page::Config(page) => page.exit_threads().await,
            Page::SendMessage(page) => page.exit_threads().await,
            Page::SignMessage(page) => page.exit_threads().await,

            Page::WalletConnect(page) => page.exit_threads().await,

            Page::Trade(page) => page.exit_threads().await,

            Page::Text(page) => page.exit_threads().await,
            Page::DevKeyCapture(page) => page.exit_threads().await,
        }
    }

    fn reload(&mut self, ss: &SharedState) -> crate::Result<()> {
        match self {
            Page::MainMenu(page) => page.reload(ss),
            Page::Setup(page) => page.reload(ss),

            Page::AddressBook(page) => page.reload(ss),
            Page::AddressBookCreate(page) => page.reload(ss),
            Page::AddressBookDisplay(page) => page.reload(ss),

            Page::Account(page) => page.reload(ss),
            Page::AccountCreate(page) => page.reload(ss),
            Page::AccountImport(page) => page.reload(ss),

            Page::Assets(page) => page.reload(ss),
            Page::AssetTransfer(page) => page.reload(ss),

            Page::Config(page) => page.reload(ss),
            Page::SendMessage(page) => page.reload(ss),
            Page::SignMessage(page) => page.reload(ss),

            Page::WalletConnect(page) => page.reload(ss),

            Page::Trade(page) => page.reload(ss),

            Page::Text(page) => page.reload(ss),
            Page::DevKeyCapture(page) => page.reload(ss),
        }
    }

    fn handle_event(
        &mut self,
        event: &Event,
        area: ratatui::prelude::Rect,
        tr: &mpsc::Sender<Event>,
        sd: &Arc<AtomicBool>,
        ss: &SharedState,
    ) -> crate::Result<HandleResult> {
        match self {
            Page::MainMenu(page) => page.handle_event(event, area, tr, sd, ss),
            Page::Setup(page) => page.handle_event(event, area, tr, sd, ss),

            Page::AddressBook(page) => page.handle_event(event, area, tr, sd, ss),
            Page::AddressBookCreate(page) => page.handle_event(event, area, tr, sd, ss),
            Page::AddressBookDisplay(page) => page.handle_event(event, area, tr, sd, ss),

            Page::Account(page) => page.handle_event(event, area, tr, sd, ss),
            Page::AccountCreate(page) => page.handle_event(event, area, tr, sd, ss),
            Page::AccountImport(page) => page.handle_event(event, area, tr, sd, ss),

            Page::Assets(page) => page.handle_event(event, area, tr, sd, ss),
            Page::AssetTransfer(page) => page.handle_event(event, area, tr, sd, ss),

            Page::Config(page) => page.handle_event(event, area, tr, sd, ss),
            Page::SendMessage(page) => page.handle_event(event, area, tr, sd, ss),
            Page::SignMessage(page) => page.handle_event(event, area, tr, sd, ss),

            Page::WalletConnect(page) => page.handle_event(event, area, tr, sd, ss),

            Page::Trade(page) => page.handle_event(event, area, tr, sd, ss),

            Page::Text(page) => page.handle_event(event, area, tr, sd, ss),
            Page::DevKeyCapture(page) => page.handle_event(event, area, tr, sd, ss),
        }
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        match self {
            Page::MainMenu(page) => page.render_component(area, buf, shared_state),
            Page::Setup(page) => page.render_component(area, buf, shared_state),

            Page::AddressBook(page) => page.render_component(area, buf, shared_state),
            Page::AddressBookCreate(page) => page.render_component(area, buf, shared_state),
            Page::AddressBookDisplay(page) => page.render_component(area, buf, shared_state),

            Page::Account(page) => page.render_component(area, buf, shared_state),
            Page::AccountCreate(page) => page.render_component(area, buf, shared_state),
            Page::AccountImport(page) => page.render_component(area, buf, shared_state),

            Page::Assets(page) => page.render_component(area, buf, shared_state),
            Page::AssetTransfer(page) => page.render_component(area, buf, shared_state),

            Page::Config(page) => page.render_component(area, buf, shared_state),
            Page::SendMessage(page) => page.render_component(area, buf, shared_state),
            Page::SignMessage(page) => page.render_component(area, buf, shared_state),

            Page::WalletConnect(page) => page.render_component(area, buf, shared_state),

            Page::Trade(page) => page.render_component(area, buf, shared_state),

            Page::Text(page) => page.render_component(area, buf, shared_state),
            Page::DevKeyCapture(page) => page.render_component(area, buf, shared_state),
        }
    }
}
