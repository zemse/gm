use std::{fmt::Display, sync::mpsc};

use alloy::primitives::Address;
use gm_ratatui_extra::filter_select_owned::FilterSelectOwned;
use ratatui::{buffer::Buffer, layout::Rect};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState,
    traits::{Actions, Component},
    AppEvent,
};
use gm_utils::{
    account::{AccountManager, AccountUtils},
    address_book::{AddressBookEntry, AddressBookStore},
    disk_storage::DiskStorageInterface,
};

use super::{
    address_book_create::AddressBookCreatePage, address_book_display::AddressBookDisplayPage, Page,
};

#[derive(Debug, PartialEq)]
pub enum AddressBookMenuItem {
    Create,
    View(AddressBookEntry),
    UnnamedOwned(Address),
    RecentlyInteracted(Address),
}

impl Display for AddressBookMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressBookMenuItem::Create => write!(f, "Create new address book entry"),
            AddressBookMenuItem::View(entry) => write!(f, "{} - {}", entry.name, entry.address),
            AddressBookMenuItem::UnnamedOwned(address) => {
                write!(f, "Self: {address}")
            }
            AddressBookMenuItem::RecentlyInteracted(address) => {
                write!(f, "Recent: {address}")
            }
        }
    }
}

impl AddressBookMenuItem {
    pub fn get_menu(
        with_create: bool,
        recently_interacted: Option<Vec<Address>>,
    ) -> crate::Result<Vec<AddressBookMenuItem>> {
        let mut entries = vec![];

        if with_create {
            entries.push(AddressBookMenuItem::Create);
        }

        // From address book
        entries.extend(
            AddressBookStore::load()?
                .list_owned()
                .into_iter()
                .map(AddressBookMenuItem::View)
                .collect::<Vec<AddressBookMenuItem>>(),
        );

        // Self accounts that do not exist in the address book
        entries.extend(
            AccountManager::get_account_list()?
                .into_iter()
                .filter(|address| {
                    !entries.iter().any(|entry| match entry {
                        AddressBookMenuItem::View(entry) => entry.address == *address,
                        _ => false,
                    })
                })
                .map(AddressBookMenuItem::UnnamedOwned)
                .collect::<Vec<AddressBookMenuItem>>(),
        );

        if let Some(recently_interacted) = recently_interacted {
            entries.extend(
                recently_interacted
                    .into_iter()
                    .filter(|address| {
                        !entries
                            .iter()
                            .any(|entry| Some(address) == entry.address().ok().as_ref())
                    })
                    .map(AddressBookMenuItem::RecentlyInteracted)
                    .collect::<Vec<AddressBookMenuItem>>(),
            );
        }

        Ok(entries)
    }

    pub fn address(&self) -> crate::Result<Address> {
        match self {
            AddressBookMenuItem::Create => Err(crate::Error::AddressBookEntryIsInvalid),
            AddressBookMenuItem::View(entry) => Ok(entry.address),
            AddressBookMenuItem::UnnamedOwned(address) => Ok(*address),
            AddressBookMenuItem::RecentlyInteracted(address) => Ok(*address),
        }
    }
}

#[derive(Debug)]
pub struct AddressBookPage {
    filter_select: FilterSelectOwned<AddressBookMenuItem>,
}

impl AddressBookPage {
    pub fn new() -> crate::Result<Self> {
        let filter_select =
            FilterSelectOwned::new(Some(AddressBookMenuItem::get_menu(true, None)?));

        Ok(Self { filter_select })
    }
}

impl Component for AddressBookPage {
    fn set_focus(&mut self, focus: bool) {
        self.filter_select.select.focus = focus;
    }

    fn reload(&mut self, _ss: &SharedState) -> crate::Result<()> {
        let fresh = Self::new()?;
        self.filter_select.set_items(fresh.filter_select.full_list);
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let mut result = Actions::default();

        self.filter_select
            .handle_event(event.input_event(), area, |ab_menu_item| {
                result.page_inserts.push(match ab_menu_item.as_ref() {
                    AddressBookMenuItem::Create => Page::AddressBookCreate(
                        AddressBookCreatePage::new(String::new(), String::new())?,
                    ),
                    AddressBookMenuItem::View(entry) => {
                        let (id, entry) = AddressBookStore::load()?
                            .find(&None, &Some(entry.address), &Some(&entry.name))?
                            .ok_or(crate::Error::AddressBookEntryNotFound(
                                entry.address,
                                entry.name.clone(),
                            ))?;
                        Page::AddressBookDisplay(AddressBookDisplayPage::new(
                            id,
                            entry.name,
                            entry.address.to_string(),
                        )?)
                    }
                    AddressBookMenuItem::UnnamedOwned(address) => Page::AddressBookCreate(
                        AddressBookCreatePage::new(String::new(), address.to_string())?,
                    ),
                    AddressBookMenuItem::RecentlyInteracted(address) => Page::AddressBookCreate(
                        AddressBookCreatePage::new(String::new(), address.to_string())?,
                    ),
                });
                Ok::<(), crate::Error>(())
            })?;

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
        self.filter_select.render(area, buf, &shared_state.theme);
        area
    }
}
