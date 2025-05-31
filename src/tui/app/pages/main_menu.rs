use std::sync::{atomic::AtomicBool, mpsc, Arc};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{layout::Rect, widgets::Widget};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{
    actions::setup::get_setup_menu,
    disk::Config,
    tui::{
        app::{widgets::select::Select, SharedState},
        events::Event,
        traits::{Component, HandleResult},
    },
    utils::cursor::Cursor,
};

use super::{
    account::AccountPage, address_book::AddressBookPage, assets::AssetsPage, config::ConfigPage,
    send_message::SendMessagePage, setup::SetupPage, sign_message::SignMessagePage, Page,
};

#[derive(Display, EnumIter)]
pub enum MainMenuItem {
    Setup,
    Assets,
    Accounts,
    AddressBook,
    SignMessage,
    SendMessage,
    Config,
}

impl MainMenuItem {
    pub fn get_page(&self) -> Page {
        match self {
            MainMenuItem::Setup => Page::Setup(SetupPage::default()),
            MainMenuItem::Assets => Page::Assets(AssetsPage::default()),
            MainMenuItem::Accounts => Page::Account(AccountPage::default()),
            MainMenuItem::AddressBook => Page::AddressBook(AddressBookPage::default()),
            MainMenuItem::SignMessage => Page::SignMessage(SignMessagePage::default()),
            MainMenuItem::SendMessage => Page::SendMessage(SendMessagePage::default()),
            MainMenuItem::Config => Page::Config(ConfigPage::default()),
        }
    }

    pub fn depends_on_current_account(&self) -> bool {
        match self {
            MainMenuItem::Setup
            | MainMenuItem::AddressBook
            | MainMenuItem::Accounts
            | MainMenuItem::Config => false,

            MainMenuItem::Assets | MainMenuItem::SignMessage | MainMenuItem::SendMessage => true,
        }
    }

    pub fn get_menu() -> Vec<MainMenuItem> {
        let mut all_options: Vec<MainMenuItem> = MainMenuItem::iter().collect();

        let setup_menu = get_setup_menu();
        if setup_menu.is_empty() {
            all_options.remove(0);
        }

        let current_account_exists = Config::current_account().is_some();
        let mut options = vec![];

        for option in all_options {
            if !option.depends_on_current_account() || current_account_exists {
                options.push(option);
            }
        }

        options
    }
}

pub struct MainMenuPage {
    pub cursor: Cursor,
    pub list: Vec<MainMenuItem>,
}

impl MainMenuPage {
    pub fn get_focussed_item(&self) -> &MainMenuItem {
        self.list
            .get(self.cursor.current)
            .expect("Invalid cursor position in MainMenuPage")
    }
}

impl Default for MainMenuPage {
    fn default() -> Self {
        Self {
            list: MainMenuItem::get_menu(),
            cursor: Cursor::default(),
        }
    }
}

impl Component for MainMenuPage {
    fn reload(&mut self) {
        let fresh = Self::default();
        self.list = fresh.list;
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        let cursor_max = self.list.len();
        self.cursor.handle(event, cursor_max);

        let mut result = HandleResult::default();
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                #[allow(clippy::single_match)]
                match key_event.code {
                    KeyCode::Enter => {
                        let mut page = self.list[self.cursor.current].get_page();
                        page.set_focus(true);
                        result.page_inserts.push(page);
                    }
                    _ => {}
                }
            }
        };

        Ok(result)
    }

    fn render_component(
        &self,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
        _shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        Select {
            list: &self.list,
            cursor: &self.cursor,
            focus: true,
            focus_style: None,
        }
        .render(area, buf);

        area
    }
}
