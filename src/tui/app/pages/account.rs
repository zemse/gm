use std::{
    fmt::Display,
    sync::{atomic::AtomicBool, mpsc, Arc},
};

use alloy::primitives::Address;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    disk::{Config, DiskInterface},
    tui::{
        app::{widgets::select::Select, SharedState},
        events::Event,
        traits::{Component, HandleResult},
    },
    utils::{
        account::{AccountManager, AccountUtils},
        cursor::Cursor,
    },
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
            AccountSelect::Existing(address) => write!(f, "{address}"),
        }
    }
}

pub struct AccountPage {
    cursor: Cursor,
    focus: bool,
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
        Self {
            cursor: Cursor::default(),
            focus: true,
            list,
        }
    }
}

impl Component for AccountPage {
    fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    fn reload(&mut self) {
        let fresh = Self::default();
        self.list = fresh.list;
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        transmitter: &mpsc::Sender<Event>,
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
                            transmitter.send(Event::ConfigUpdate)?;
                            result.page_pops = 1;
                        }
                    },
                    _ => {}
                }
            }
        };

        Ok(result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, _shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        Select {
            list: &self.list,
            cursor: &self.cursor,
            focus: self.focus,
            focus_style: None,
        }
        .render(area, buf);
        area
    }
}
