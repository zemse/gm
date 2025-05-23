use std::sync::{atomic::AtomicBool, mpsc, Arc};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::Widget;

use crate::{
    actions::Action,
    tui::{
        app::{widgets::select::Select, SharedState},
        events::Event,
        traits::{Component, HandleResult},
    },
    utils::cursor::Cursor,
};

use super::{
    account::AccountPage, address_book::AddressBookPage, assets::AssetsPage, config::ConfigPage,
    send_message::SendMessagePage, sign_message::SignMessagePage, transaction::TransactionPage,
    Page,
};

pub struct MainMenuPage {
    cursor: Cursor,
    list: Vec<Action>,
}

impl Default for MainMenuPage {
    fn default() -> Self {
        Self {
            list: Action::get_menu(),
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
                        Action::Setup => todo!(),
                        Action::AddressBook { .. } => {
                            result
                                .page_inserts
                                .push(Page::AddressBook(AddressBookPage::default()));
                        }
                        Action::Assets => result
                            .page_inserts
                            .push(Page::Assets(AssetsPage::default())),
                        Action::Account { .. } => {
                            result
                                .page_inserts
                                .push(Page::Account(AccountPage::default()));
                        }
                        Action::Transaction { .. } => {
                            result.page_inserts.push(Page::Transaction(TransactionPage))
                        }
                        Action::SignMessage { .. } => {
                            result.page_inserts.push(Page::SignMessage(SignMessagePage))
                        }
                        Action::SendMessage { .. } => result
                            .page_inserts
                            .push(Page::SendMessage(SendMessagePage::default())),
                        Action::Config { .. } => result
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
        _: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        Select {
            list: &self.list,
            cursor: &self.cursor,
        }
        .render(area, buf);

        area
    }
}
