use std::sync::mpsc;

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
use transaction::TransactionPage;

use crate::tui::{
    events::Event,
    traits::{Component, HandleResult},
};

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
pub mod transaction;

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
}

impl Page {
    pub fn is_full_screen(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match self {
            Page::AddressBookDisplay(_) => true,
            _ => false,
        }
    }

    pub fn is_main_menu(&self) -> bool {
        matches!(self, Page::MainMenu(_))
    }
}

impl Component for Page {
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
        }
    }

    fn handle_event(
        &mut self,
        event: &Event,
        tr: &mpsc::Sender<Event>,
    ) -> crate::Result<HandleResult> {
        match self {
            Page::MainMenu(page) => page.handle_event(event, tr),
            Page::Setup(page) => page.handle_event(event, tr),

            Page::AddressBook(page) => page.handle_event(event, tr),
            Page::AddressBookCreate(page) => page.handle_event(event, tr),
            Page::AddressBookDisplay(page) => page.handle_event(event, tr),

            Page::Account(page) => page.handle_event(event, tr),
            Page::AccountCreate(page) => page.handle_event(event, tr),
            Page::AccountImport(page) => page.handle_event(event, tr),

            Page::Assets(page) => page.handle_event(event, tr),
            Page::Config(page) => page.handle_event(event, tr),
            Page::SendMessage(page) => page.handle_event(event, tr),
            Page::SignMessage(page) => page.handle_event(event, tr),
            Page::Transaction(page) => page.handle_event(event, tr),
        }
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        match self {
            Page::MainMenu(page) => page.render_component(area, buf),
            Page::Setup(page) => page.render_component(area, buf),

            Page::AddressBook(page) => page.render_component(area, buf),
            Page::AddressBookCreate(page) => page.render_component(area, buf),
            Page::AddressBookDisplay(page) => page.render_component(area, buf),

            Page::Account(page) => page.render_component(area, buf),
            Page::AccountCreate(page) => page.render_component(area, buf),
            Page::AccountImport(page) => page.render_component(area, buf),

            Page::Assets(page) => page.render_component(area, buf),
            Page::Config(page) => page.render_component(area, buf),
            Page::SendMessage(page) => page.render_component(area, buf),
            Page::SignMessage(page) => page.render_component(area, buf),
            Page::Transaction(page) => page.render_component(area, buf),
        }
    }
}
