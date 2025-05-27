use std::sync::{atomic::AtomicBool, mpsc, Arc};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::Widget;
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{
    actions::setup::get_setup_menu,
    disk::Config,
    tui::{
        app::{widgets::select::Select, Focus, SharedState},
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
pub enum MainMenuItems {
    Setup,
    Assets,
    Accounts,
    AddressBook,
    SignMessage,
    SendMessage,
    Config,
}

impl MainMenuItems {
    pub fn depends_on_current_account(&self) -> bool {
        match self {
            MainMenuItems::Setup
            | MainMenuItems::AddressBook
            | MainMenuItems::Accounts
            | MainMenuItems::Config => false,

            MainMenuItems::Assets | MainMenuItems::SignMessage | MainMenuItems::SendMessage => true,
        }
    }

    pub fn get_menu() -> Vec<MainMenuItems> {
        let mut all_options: Vec<MainMenuItems> = MainMenuItems::iter().collect();

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
    cursor: Cursor,
    list: Vec<MainMenuItems>,
}

impl Default for MainMenuPage {
    fn default() -> Self {
        Self {
            list: MainMenuItems::get_menu(),
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
                    KeyCode::Enter => match &self.list[self.cursor.current] {
                        &MainMenuItems::Setup => {
                            result.page_inserts.push(Page::Setup(SetupPage::default()))
                        }
                        MainMenuItems::AddressBook => {
                            result
                                .page_inserts
                                .push(Page::AddressBook(AddressBookPage::default()));
                        }
                        MainMenuItems::Assets => result
                            .page_inserts
                            .push(Page::Assets(AssetsPage::default())),
                        MainMenuItems::Accounts => {
                            result
                                .page_inserts
                                .push(Page::Account(AccountPage::default()));
                        }
                        MainMenuItems::SignMessage => {
                            result.page_inserts.push(Page::SignMessage(SignMessagePage))
                        }
                        MainMenuItems::SendMessage => result
                            .page_inserts
                            .push(Page::SendMessage(SendMessagePage::default())),
                        MainMenuItems::Config => result
                            .page_inserts
                            .push(Page::Config(ConfigPage::default())),
                    },
                    _ => {}
                }
            }
        };

        Ok(result)
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
        Select {
            list: &self.list,
            cursor: &self.cursor,
            focus: shared_state.focus == Focus::Main,
        }
        .render(area, buf);

        area
    }
}
