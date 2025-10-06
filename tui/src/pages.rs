use std::sync::mpsc;

use account::AccountPage;
use account_create::AccountCreatePage;
use account_import::AccountImportPage;
use address_book::AddressBookPage;
use address_book_create::AddressBookCreatePage;
use address_book_display::AddressBookDisplayPage;
use asset_transfer::AssetTransferPage;
use assets::AssetsPage;
use complete_setup::CompleteSetupPage;
use config::ConfigPage;
use dev_key_capture::DevKeyCapturePage;
use ratatui::{buffer::Buffer, layout::Rect};
use send_message::SendMessagePage;
use sign_message::SignMessagePage;
use text::TextPage;
use tokio_util::sync::CancellationToken;
use trade::TradePage;
use walletconnect::WalletConnectPage;

use crate::{
    app::SharedState,
    pages::{
        network::NetworkPage, network_create::NetworkCreatePage, shell::ShellPage,
        token::TokenPage, token_create::TokenCreatePage,
    },
    traits::{Actions, Component},
    AppEvent,
};

pub mod account;
pub mod account_create;
pub mod account_import;
pub mod address_book;
pub mod address_book_create;
pub mod address_book_display;
pub mod asset_transfer;
pub mod assets;
pub mod complete_setup;
pub mod config;
pub mod dev_key_capture;
pub mod footer;
pub mod invite_popup;
pub mod network;
pub mod network_create;
pub mod send_message;
pub mod shell;
pub mod sign_message;
pub mod sign_popup;
pub mod sign_typed_data_popup;
pub mod text;
pub mod title;
pub mod token;
pub mod token_create;
pub mod trade;
pub mod tx_popup;
pub mod walletconnect;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Page {
    CompleteSetup(CompleteSetupPage),

    Account(AccountPage),
    AccountCreate(AccountCreatePage),
    AccountImport(AccountImportPage),

    AddressBook(AddressBookPage),
    AddressBookCreate(AddressBookCreatePage),
    AddressBookDisplay(AddressBookDisplayPage),

    Network(NetworkPage),
    NetworkCreate(NetworkCreatePage),
    Token(TokenPage),
    TokenCreate(TokenCreatePage),

    Assets(AssetsPage),
    AssetTransfer(AssetTransferPage),

    Config(ConfigPage),
    SendMessage(SendMessagePage),
    SignMessage(SignMessagePage),

    WalletConnect(WalletConnectPage),

    Trade(TradePage),

    Text(TextPage),
    DevKeyCapture(DevKeyCapturePage),

    Shell(ShellPage),
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
}

impl Component for Page {
    fn set_focus(&mut self, focus: bool) {
        match self {
            Page::CompleteSetup(page) => page.set_focus(focus),

            Page::AddressBook(page) => page.set_focus(focus),
            Page::AddressBookCreate(page) => page.set_focus(focus),
            Page::AddressBookDisplay(page) => page.set_focus(focus),

            Page::Account(page) => page.set_focus(focus),
            Page::AccountCreate(page) => page.set_focus(focus),
            Page::AccountImport(page) => page.set_focus(focus),

            Page::Network(page) => page.set_focus(focus),
            Page::NetworkCreate(page) => page.set_focus(focus),
            Page::Token(page) => page.set_focus(focus),
            Page::TokenCreate(page) => page.set_focus(focus),

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

            Page::Shell(page) => page.set_focus(focus),
        }
    }

    fn set_cursor(&mut self, cursor: usize) {
        match self {
            Page::CompleteSetup(page) => page.set_cursor(cursor),

            Page::AddressBook(page) => page.set_cursor(cursor),
            Page::AddressBookCreate(page) => page.set_cursor(cursor),
            Page::AddressBookDisplay(page) => page.set_cursor(cursor),

            Page::Account(page) => page.set_cursor(cursor),
            Page::AccountCreate(page) => page.set_cursor(cursor),
            Page::AccountImport(page) => page.set_cursor(cursor),

            Page::Network(page) => page.set_cursor(cursor),
            Page::NetworkCreate(page) => page.set_cursor(cursor),
            Page::Token(page) => page.set_cursor(cursor),
            Page::TokenCreate(page) => page.set_cursor(cursor),

            Page::Assets(page) => page.set_cursor(cursor),
            Page::AssetTransfer(page) => page.set_cursor(cursor),

            Page::Config(page) => page.set_cursor(cursor),
            Page::SendMessage(page) => page.set_cursor(cursor),
            Page::SignMessage(page) => page.set_cursor(cursor),

            Page::WalletConnect(page) => page.set_cursor(cursor),

            Page::Trade(page) => page.set_cursor(cursor),

            Page::Text(page) => page.set_cursor(cursor),
            Page::DevKeyCapture(page) => page.set_cursor(cursor),

            Page::Shell(page) => page.set_cursor(cursor),
        }
    }

    fn get_cursor(&self) -> Option<usize> {
        match self {
            Page::CompleteSetup(page) => page.get_cursor(),

            Page::AddressBook(page) => page.get_cursor(),
            Page::AddressBookCreate(page) => page.get_cursor(),
            Page::AddressBookDisplay(page) => page.get_cursor(),

            Page::Account(page) => page.get_cursor(),
            Page::AccountCreate(page) => page.get_cursor(),
            Page::AccountImport(page) => page.get_cursor(),

            Page::Network(page) => page.get_cursor(),
            Page::NetworkCreate(page) => page.get_cursor(),
            Page::Token(page) => page.get_cursor(),
            Page::TokenCreate(page) => page.get_cursor(),

            Page::Assets(page) => page.get_cursor(),
            Page::AssetTransfer(page) => page.get_cursor(),

            Page::Config(page) => page.get_cursor(),
            Page::SendMessage(page) => page.get_cursor(),
            Page::SignMessage(page) => page.get_cursor(),

            Page::WalletConnect(page) => page.get_cursor(),

            Page::Trade(page) => page.get_cursor(),

            Page::Text(page) => page.get_cursor(),
            Page::DevKeyCapture(page) => page.get_cursor(),

            Page::Shell(page) => page.get_cursor(),
        }
    }

    async fn exit_threads(&mut self) {
        match self {
            Page::CompleteSetup(page) => page.exit_threads().await,

            Page::AddressBook(page) => page.exit_threads().await,
            Page::AddressBookCreate(page) => page.exit_threads().await,
            Page::AddressBookDisplay(page) => page.exit_threads().await,

            Page::Network(page) => page.exit_threads().await,
            Page::NetworkCreate(page) => page.exit_threads().await,
            Page::Token(page) => page.exit_threads().await,
            Page::TokenCreate(page) => page.exit_threads().await,

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

            Page::Shell(page) => page.exit_threads().await,
        }
    }

    fn reload(&mut self, ss: &SharedState) -> crate::Result<()> {
        match self {
            Page::CompleteSetup(page) => page.reload(ss),

            Page::AddressBook(page) => page.reload(ss),
            Page::AddressBookCreate(page) => page.reload(ss),
            Page::AddressBookDisplay(page) => page.reload(ss),

            Page::Network(page) => page.reload(ss),
            Page::NetworkCreate(page) => page.reload(ss),
            Page::Token(page) => page.reload(ss),
            Page::TokenCreate(page) => page.reload(ss),

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

            Page::Shell(page) => page.reload(ss),
        }
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        tr: &mpsc::Sender<AppEvent>,
        sd: &CancellationToken,
        ss: &SharedState,
    ) -> crate::Result<Actions> {
        match self {
            Page::CompleteSetup(page) => page.handle_event(event, area, tr, sd, ss),

            Page::AddressBook(page) => page.handle_event(event, area, tr, sd, ss),
            Page::AddressBookCreate(page) => page.handle_event(event, area, tr, sd, ss),
            Page::AddressBookDisplay(page) => page.handle_event(event, area, tr, sd, ss),

            Page::Network(page) => page.handle_event(event, area, tr, sd, ss),
            Page::NetworkCreate(page) => page.handle_event(event, area, tr, sd, ss),
            Page::Token(page) => page.handle_event(event, area, tr, sd, ss),
            Page::TokenCreate(page) => page.handle_event(event, area, tr, sd, ss),

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

            Page::Shell(page) => page.handle_event(event, area, tr, sd, ss),
        }
    }

    fn render_component(
        &self,
        area: Rect,
        popup_area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        match self {
            Page::CompleteSetup(page) => page.render_component(area, popup_area, buf, shared_state),

            Page::AddressBook(page) => page.render_component(area, popup_area, buf, shared_state),
            Page::AddressBookCreate(page) => {
                page.render_component(area, popup_area, buf, shared_state)
            }
            Page::AddressBookDisplay(page) => {
                page.render_component(area, popup_area, buf, shared_state)
            }

            Page::Network(page) => page.render_component(area, popup_area, buf, shared_state),
            Page::NetworkCreate(page) => page.render_component(area, popup_area, buf, shared_state),
            Page::Token(page) => page.render_component(area, popup_area, buf, shared_state),
            Page::TokenCreate(page) => page.render_component(area, popup_area, buf, shared_state),

            Page::Account(page) => page.render_component(area, popup_area, buf, shared_state),
            Page::AccountCreate(page) => page.render_component(area, popup_area, buf, shared_state),
            Page::AccountImport(page) => page.render_component(area, popup_area, buf, shared_state),

            Page::Assets(page) => page.render_component(area, popup_area, buf, shared_state),
            Page::AssetTransfer(page) => page.render_component(area, popup_area, buf, shared_state),

            Page::Config(page) => page.render_component(area, popup_area, buf, shared_state),
            Page::SendMessage(page) => page.render_component(area, popup_area, buf, shared_state),
            Page::SignMessage(page) => page.render_component(area, popup_area, buf, shared_state),

            Page::WalletConnect(page) => page.render_component(area, popup_area, buf, shared_state),

            Page::Trade(page) => page.render_component(area, popup_area, buf, shared_state),

            Page::Text(page) => page.render_component(area, popup_area, buf, shared_state),
            Page::DevKeyCapture(page) => page.render_component(area, popup_area, buf, shared_state),

            Page::Shell(page) => page.render_component(area, popup_area, buf, shared_state),
        }
    }
}
