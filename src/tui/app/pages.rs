use std::sync::{atomic::AtomicBool, mpsc, Arc};

use account::AccountPage;
use account_create::AccountCreatePage;
use account_import::AccountImportPage;
use address_book::AddressBookPage;
use address_book_create::AddressBookCreatePage;
use address_book_display::AddressBookDisplayPage;
use assets::AssetsPage;
use config::ConfigPage;
use main_menu::MainMenuPage;
use send_message::SendMessagePage;
use setup::SetupPage;
use sign_message::SignMessagePage;
use trade::TradePage;
use transaction::TransactionPage;

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
pub mod assets;
pub mod config;
pub mod main_menu;
pub mod send_message;
pub mod setup;
pub mod sign_message;
pub mod trade;
pub mod transaction;

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
    Config(ConfigPage),
    SendMessage(SendMessagePage),
    SignMessage(SignMessagePage),
    Transaction(TransactionPage),

    Trade(TradePage),
}

impl Page {
    pub fn is_full_screen(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match self {
            Page::AddressBookDisplay(_) => true,
            Page::Trade(_) => true,
            _ => false,
        }
    }

    pub fn is_main_menu(&self) -> bool {
        matches!(self, Page::MainMenu(_))
    }
}

impl Component for Page {
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
            Page::Config(page) => page.exit_threads().await,
            Page::SendMessage(page) => page.exit_threads().await,
            Page::SignMessage(page) => page.exit_threads().await,
            Page::Transaction(page) => page.exit_threads().await,

            Page::Trade(page) => page.exit_threads().await,
        }
    }

    fn reload(&mut self) {
        match self {
            Page::MainMenu(page) => page.reload(),
            Page::Setup(page) => page.reload(),

            Page::AddressBook(page) => page.reload(),
            Page::AddressBookCreate(page) => page.reload(),
            Page::AddressBookDisplay(page) => page.reload(),

            Page::Account(page) => page.reload(),
            Page::AccountCreate(page) => page.reload(),
            Page::AccountImport(page) => page.reload(),

            Page::Assets(page) => page.reload(),
            Page::Config(page) => page.reload(),
            Page::SendMessage(page) => page.reload(),
            Page::SignMessage(page) => page.reload(),
            Page::Transaction(page) => page.reload(),

            Page::Trade(page) => page.reload(),
        }
    }

    fn handle_event(
        &mut self,
        event: &Event,
        tr: &mpsc::Sender<Event>,
        sd: &Arc<AtomicBool>,
    ) -> crate::Result<HandleResult> {
        match self {
            Page::MainMenu(page) => page.handle_event(event, tr, sd),
            Page::Setup(page) => page.handle_event(event, tr, sd),

            Page::AddressBook(page) => page.handle_event(event, tr, sd),
            Page::AddressBookCreate(page) => page.handle_event(event, tr, sd),
            Page::AddressBookDisplay(page) => page.handle_event(event, tr, sd),

            Page::Account(page) => page.handle_event(event, tr, sd),
            Page::AccountCreate(page) => page.handle_event(event, tr, sd),
            Page::AccountImport(page) => page.handle_event(event, tr, sd),

            Page::Assets(page) => page.handle_event(event, tr, sd),
            Page::Config(page) => page.handle_event(event, tr, sd),
            Page::SendMessage(page) => page.handle_event(event, tr, sd),
            Page::SignMessage(page) => page.handle_event(event, tr, sd),
            Page::Transaction(page) => page.handle_event(event, tr, sd),

            Page::Trade(page) => page.handle_event(event, tr, sd),
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
            Page::Config(page) => page.render_component(area, buf, shared_state),
            Page::SendMessage(page) => page.render_component(area, buf, shared_state),
            Page::SignMessage(page) => page.render_component(area, buf, shared_state),
            Page::Transaction(page) => page.render_component(area, buf, shared_state),

            Page::Trade(page) => page.render_component(area, buf, shared_state),
        }
    }
}
