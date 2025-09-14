use std::{
    fmt::Display,
    sync::{atomic::AtomicBool, mpsc, Arc},
};

use crate::{
    app::SharedState,
    events::Event,
    traits::{Actions, Component},
};
use alloy::primitives::Address;
use gm_ratatui_extra::{cursor::Cursor, select::Select, thematize::Thematize};
use gm_utils::{
    account::{AccountManager, AccountUtils},
    disk::{Config, DiskInterface},
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEventKind},
    layout::Rect,
    widgets::Widget,
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

impl AccountPage {
    pub fn new() -> crate::Result<Self> {
        let mut list = vec![AccountSelect::Create, AccountSelect::Import];
        list.extend(
            AccountManager::get_account_list()?
                .into_iter()
                .map(AccountSelect::Existing)
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
    ) -> crate::Result<Actions> {
        let cursor_max = self.list.len();
        self.cursor.handle(event.key_event(), cursor_max);

        let mut result = Actions::default();
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
            focus_style: shared_state.theme.select_focused(),
        }
        .render(area, buf);
        area
    }
}
