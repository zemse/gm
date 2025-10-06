use std::{fmt::Display, sync::mpsc};

use crate::{
    app::SharedState,
    traits::{Actions, Component},
    AppEvent,
};
use alloy::primitives::Address;
use gm_ratatui_extra::{extensions::ThemedWidget, select_owned::SelectOwned};
use gm_utils::{
    account::{AccountManager, AccountUtils},
    config::Config,
};
use ratatui::{buffer::Buffer, layout::Rect};
use tokio_util::sync::CancellationToken;

use super::{account_create::AccountCreatePage, account_import::AccountImportPage, Page};

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub struct AccountPage {
    select: SelectOwned<AccountSelect>,
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
        Ok(AccountPage {
            select: SelectOwned::new(Some(list), false),
        })
    }
}

impl Component for AccountPage {
    fn set_focus(&mut self, focus: bool) {
        self.select.focus = focus;
    }

    fn reload(&mut self, _ss: &SharedState) -> crate::Result<()> {
        let fresh = Self::new()?;
        self.select.list = fresh.select.list;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        _area: Rect,
        transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let mut result = Actions::default();

        self.select.handle_event(
            event.input_event(),
            _area,
            |account_select| {
                match account_select {
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
                        Config::set_current_account(*address)?;
                        transmitter.send(AppEvent::AccountChange(*address))?;
                        transmitter.send(AppEvent::ConfigUpdate)?;
                        result.page_pop = true;
                    }
                }
                Ok::<(), crate::Error>(())
            },
            |_| Ok(()),
        )?;

        Ok(result)
    }

    fn render_component(
        &self,
        area: Rect,
        _popup_area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        self.select.render(area, buf, &shared_state.theme);
        area
    }
}
