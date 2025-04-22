use std::{fmt::Display, sync::mpsc};

use alloy::primitives::Address;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    disk::{Config, DiskInterface},
    tui::{
        app::widgets::select::Select,
        events::Event,
        traits::{Component, HandleResult},
    },
    utils::account::{AccountManager, AccountUtils},
};

use super::{account_create::AccountCreatePage, account_import::AccountImportPage, Page};

enum AccountSelect {
    Create,
    Import,
    Existing(Address),
}

impl Display for AccountSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountSelect::Create => write!(f, "Create new wallet"),
            AccountSelect::Import => write!(f, "Import existing wallet"),
            AccountSelect::Existing(address) => write!(f, "{}", address),
        }
    }
}

pub struct AccountPage {
    cursor: usize,
    list: Vec<AccountSelect>,
}

impl Default for AccountPage {
    fn default() -> Self {
        let mut list = vec![AccountSelect::Create, AccountSelect::Import];
        list.extend(
            AccountManager::get_account_list()
                .into_iter()
                .map(AccountSelect::Existing)
                .collect::<Vec<_>>(),
        );
        Self { cursor: 0, list }
    }
}

impl Component for AccountPage {
    fn reload(&mut self) {
        let fresh = Self::default();
        self.list = fresh.list;
    }

    fn handle_event(
        &mut self,
        event: &Event,
        transmitter: &mpsc::Sender<Event>,
    ) -> crate::Result<HandleResult> {
        let cursor_max = self.list.len();

        let mut result = HandleResult::default();
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Up => {
                        self.cursor = (self.cursor + cursor_max - 1) % cursor_max;
                    }
                    KeyCode::Down => {
                        self.cursor = (self.cursor + 1) % cursor_max;
                    }
                    KeyCode::Enter => match &self.list[self.cursor] {
                        AccountSelect::Create => {
                            result
                                .page_inserts
                                .push(Page::AccountCreate(AccountCreatePage::default()));
                        }
                        AccountSelect::Import => {
                            result
                                .page_inserts
                                .push(Page::AccountImport(AccountImportPage::default()));
                        }
                        AccountSelect::Existing(address) => {
                            let mut config = Config::load();
                            config.current_account = Some(*address);
                            config.save();
                            transmitter.send(Event::AccountChange(*address))?;
                            result.page_pops = 1;
                        }
                    },
                    _ => {}
                }
            }
        };

        Ok(result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer) -> Rect
    where
        Self: Sized,
    {
        Select {
            list: &self.list,
            cursor: Some(&self.cursor),
        }
        .render(area, buf);
        area
    }
}
