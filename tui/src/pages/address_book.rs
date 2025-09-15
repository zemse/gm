use std::{
    fmt::Display,
    sync::{atomic::AtomicBool, mpsc, Arc},
};

use alloy::primitives::Address;
use gm_ratatui_extra::{cursor::Cursor, filter_select::FilterSelect, thematize::Thematize};
use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind},
    widgets::Widget,
};

use crate::{
    app::SharedState,
    events::Event,
    traits::{Actions, Component},
};
use gm_utils::{
    account::{AccountManager, AccountUtils},
    address_book::{AddressBookEntry, AddressBookStore},
    disk_storage::DiskStorageInterface,
};

use super::{
    address_book_create::AddressBookCreatePage, address_book_display::AddressBookDisplayPage, Page,
};

#[derive(Debug)]
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

pub struct AddressBookPage {
    full_list: Vec<AddressBookMenuItem>,
    search_string: String,
    cursor: Cursor,
    focus: bool,
}

impl AddressBookPage {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            full_list: AddressBookMenuItem::get_menu(true, None)?,
            search_string: String::new(),
            cursor: Cursor::default(),
            focus: true,
        })
    }
}

impl Component for AddressBookPage {
    fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    fn reload(&mut self, _ss: &SharedState) -> crate::Result<()> {
        let fresh = Self::new()?;
        self.full_list = fresh.full_list;
        Ok(())
    }

    fn text_input_mut(&mut self) -> Option<&mut String> {
        Some(&mut self.search_string)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: ratatui::prelude::Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let list: Vec<&AddressBookMenuItem> = self
            .full_list
            .iter()
            .filter(|entry| format!("{entry}").contains(self.search_string.as_str()))
            .collect();

        let cursor_max = list.len();
        self.cursor.handle(event.key_event(), cursor_max);

        let mut result = Actions::default();
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Char(char) => {
                        if let Some(text_input) = self.text_input_mut() {
                            text_input.push(char);
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(text_input) = self.text_input_mut() {
                            text_input.pop();
                        }
                    }
                    KeyCode::Enter => result.page_inserts.push(match &list[self.cursor.current] {
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
                        AddressBookMenuItem::RecentlyInteracted(address) => {
                            Page::AddressBookCreate(AddressBookCreatePage::new(
                                String::new(),
                                address.to_string(),
                            )?)
                        }
                    }),
                    _ => {}
                }
            }
        }

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
        FilterSelect {
            full_list: &self.full_list,
            cursor: &self.cursor,
            search_string: &self.search_string,
            focus: self.focus,
            focus_style: shared_state.theme.select_focused(),
        }
        .render(area, buf);
        area
    }
}
