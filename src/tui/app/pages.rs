use address_book::AddressBookPage;
use address_book_create::AddressBookCreatePage;
use address_book_display::AddressBookDisplayPage;
use main_menu::MainMenuPage;

use crate::tui::{events::Event, traits::Component};

pub mod address_book;
pub mod address_book_create;
pub mod address_book_display;
pub mod main_menu;

pub enum Page {
    MainMenu(MainMenuPage),
    AddressBook(AddressBookPage),
    AddressBookCreate(AddressBookCreatePage),
    AddressBookDisplay(AddressBookDisplayPage),
}

impl Page {
    pub fn is_full_screen(&self) -> bool {
        match self {
            Page::MainMenu(_) | Page::AddressBook(_) | Page::AddressBookCreate(_) => false,
            Page::AddressBookDisplay(_) => true,
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
            Page::AddressBook(page) => page.reload(),
            Page::AddressBookCreate(page) => page.reload(),
            Page::AddressBookDisplay(page) => page.reload(),
        }
    }

    fn handle_event(&mut self, event: &Event) -> crate::tui::traits::HandleResult {
        match self {
            Page::MainMenu(page) => page.handle_event(event),
            Page::AddressBook(page) => page.handle_event(event),
            Page::AddressBookCreate(page) => page.handle_event(event),
            Page::AddressBookDisplay(page) => page.handle_event(event),
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
            Page::AddressBook(page) => page.render_component(area, buf),
            Page::AddressBookCreate(page) => page.render_component(area, buf),
            Page::AddressBookDisplay(page) => page.render_component(area, buf),
        }
    }
}
