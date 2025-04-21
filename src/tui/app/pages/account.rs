use std::{fmt::Display, sync::mpsc};

use alloy::primitives::Address;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    actions::account,
    disk::{Config, DiskInterface},
    tui::{
        app::widgets::select::Select,
        events::Event,
        traits::{Component, HandleResult},
    },
};

use super::{account_create::AccountCreatePage, Page};

enum AccountSelect {
    CreateWallet,
    ExistingWallet(Address),
}

impl Display for AccountSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountSelect::CreateWallet => write!(f, "Create new wallet"),
            AccountSelect::ExistingWallet(address) => write!(f, "{}", address),
        }
    }
}

pub struct AccountPage {
    cursor: usize,
    list: Vec<AccountSelect>,
}

impl Default for AccountPage {
    fn default() -> Self {
        let mut list = vec![AccountSelect::CreateWallet];
        list.extend(
            account::list_of_wallets()
                .into_iter()
                .map(AccountSelect::ExistingWallet)
                .collect::<Vec<_>>(),
        );
        Self { cursor: 0, list }
    }
}

impl Component for AccountPage {
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
                        AccountSelect::CreateWallet => {
                            result
                                .page_inserts
                                .push(Page::AccountCreate(AccountCreatePage));
                        }
                        AccountSelect::ExistingWallet(address) => {
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
