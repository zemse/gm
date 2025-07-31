use std::{
    fmt::Display,
    sync::{atomic::AtomicBool, mpsc, Arc},
};

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

use crossterm::event::{KeyCode, KeyEventKind};
use fusion_plus_sdk::multichain_address::MultichainAddress;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::{account_create::AccountCreatePage, account_import::AccountImportPage, Page};

enum AccountSelect {
    Create,
    Import,
    Existing(MultichainAddress),
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

impl AccountPage {
    pub fn new() -> crate::Result<Self> {
        let mut list = vec![AccountSelect::Create, AccountSelect::Import];
        list.extend(
            AccountManager::get_account_list()?
                .into_iter()
                .map(|addr| AccountSelect::Existing(addr.into()))
                .collect::<Vec<_>>(),
        );
        Ok(Self {
            cursor: Cursor::default(),
            focus: true,
            list,
        })
    }
}

impl Component for AccountPage {
    fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    fn reload(&mut self, _ss: &SharedState) -> crate::Result<()> {
        let fresh = Self::new()?;
        self.list = fresh.list;
        Ok(())
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
                            let mut config = Config::load()?;
                            config.current_account = Some(*address);
                            config.save()?;
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

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        Select {
            list: &self.list,
            cursor: &self.cursor,
            focus: self.focus,
            focus_style: shared_state.theme.select(),
        }
        .render(area, buf);
        area
    }
}
