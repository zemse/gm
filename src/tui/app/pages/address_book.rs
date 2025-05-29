use std::{
    fmt::Display,
    sync::{atomic::AtomicBool, mpsc, Arc},
};

use alloy::primitives::Address;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::Widget;

use crate::{
    disk::{AddressBook, AddressBookEntry, DiskInterface},
    tui::{
        app::{widgets::filter_select::FilterSelect, Focus, SharedState},
        events::Event,
        traits::{Component, HandleResult},
    },
    utils::{
        account::{AccountManager, AccountUtils},
        cursor::Cursor,
    },
};

use super::{
    address_book_create::AddressBookCreatePage, address_book_display::AddressBookDisplayPage, Page,
};

#[derive(Debug)]
pub enum AddressBookMenuItem {
    Create,
    View(AddressBookEntry),
    UnnamedOwned(Address),
}

impl Display for AddressBookMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressBookMenuItem::Create => write!(f, "Create new address book entry"),
            AddressBookMenuItem::View(entry) => write!(f, "{} - {}", entry.name, entry.address),
            AddressBookMenuItem::UnnamedOwned(address) => {
                write!(f, "Un-named: {}", address)
            }
        }
    }
}

impl AddressBookMenuItem {
    pub fn get_menu(with_create: bool) -> Vec<AddressBookMenuItem> {
        let mut entries = vec![];

        if with_create {
            entries.push(AddressBookMenuItem::Create);
        }

        // From address book
        entries.extend(
            AddressBook::load()
                .list_owned()
                .into_iter()
                .map(AddressBookMenuItem::View)
                .collect::<Vec<AddressBookMenuItem>>(),
        );

        // Self accounts that do not exist in the address book
        entries.extend(
            AccountManager::get_account_list()
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

        entries
    }

    // Must only be used if you are sure that the list will not contain Create
    pub fn address_unwrap(&self) -> Address {
        match self {
            AddressBookMenuItem::Create => {
                unreachable!("AddressBookMenuItem::Create entry must not be present")
            }
            AddressBookMenuItem::View(entry) => entry.address,
            AddressBookMenuItem::UnnamedOwned(address) => *address,
        }
    }
}

pub struct AddressBookPage {
    full_list: Vec<AddressBookMenuItem>,
    search_string: String,
    cursor: Cursor,
}

impl Default for AddressBookPage {
    fn default() -> Self {
        Self {
            full_list: AddressBookMenuItem::get_menu(true),
            search_string: String::new(),
            cursor: Cursor::default(),
        }
    }
}

impl Component for AddressBookPage {
    fn reload(&mut self) {
        let fresh = Self::default();
        self.full_list = fresh.full_list;
    }

    fn text_input_mut(&mut self) -> Option<&mut String> {
        Some(&mut self.search_string)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        let list: Vec<&AddressBookMenuItem> = self
            .full_list
            .iter()
            .filter(|entry| format!("{entry}").contains(self.search_string.as_str()))
            .collect();

        let cursor_max = list.len();
        self.cursor.handle(event, cursor_max);

        let mut result = HandleResult::default();
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
                            AddressBookCreatePage::new(String::new(), String::new()),
                        ),
                        AddressBookMenuItem::View(entry) => {
                            let (id, entry) = AddressBook::load()
                                .find(&None, &Some(entry.address), &Some(&entry.name))
                                .expect("entry not found");
                            Page::AddressBookDisplay(AddressBookDisplayPage::new(
                                id,
                                entry.name,
                                entry.address.to_string(),
                            ))
                        }
                        AddressBookMenuItem::UnnamedOwned(address) => Page::AddressBookCreate(
                            AddressBookCreatePage::new(String::new(), address.to_string()),
                        ),
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
            focus: shared_state.focus == Focus::Main,
            focus_style: None,
        }
        .render(area, buf);
        area
    }
}
