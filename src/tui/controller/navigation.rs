use alloy::primitives::Address;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::marker::PhantomData;
use strum_macros::Display;

use crate::{
    actions::{address_book::AddressBookActions, Action},
    disk::{AddressBook, AddressBookEntry, DiskInterface},
};

#[derive(Display, Debug)]
pub enum Page {
    MainMenu {
        list: Vec<Action>,
        cursor: usize,
    },
    AddressBook {
        full_list: Vec<AddressBookActions>,
        search_string: String,
        cursor: usize,
    },
    AddressBookCreateNewEntry {
        cursor: usize,
        name: String,
        address: String,
        error: Option<String>,
    },
    AddressBookDisplayEntry {
        cursor: usize,
        edit: bool,
        id: usize,
        name: String,
        address: Address,
    },
}

#[derive(Debug)]
pub struct Navigation<'a> {
    pub pages: Vec<Page>,
    _marker: PhantomData<&'a ()>,
}

impl Default for Navigation<'_> {
    fn default() -> Self {
        let list = Action::get_menu();
        Self {
            pages: vec![Page::MainMenu { list, cursor: 0 }],
            _marker: PhantomData,
        }
    }
}

impl Navigation<'_> {
    pub fn handle(&mut self, key_event: KeyEvent) {
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
                KeyCode::Esc => {
                    self.pages.pop();
                }
                KeyCode::Enter => {
                    // go to next menu
                    self.enter();
                }
                KeyCode::Up => {
                    self.up();
                }
                KeyCode::Down => {
                    self.down();
                }
                // TODO
                // KeyCode::Left => {}
                // KeyCode::Right => {}
                _ => {}
            }
        }
    }

    pub fn text_input_mut(&mut self) -> Option<&mut String> {
        if let Some(page) = self.current_page_mut() {
            match page {
                Page::AddressBook { search_string, .. } => Some(search_string),
                Page::AddressBookCreateNewEntry {
                    cursor,
                    name,
                    address,
                    ..
                } => match cursor {
                    0 => Some(name),
                    1 => Some(address),
                    _ => None,
                },
                Page::AddressBookDisplayEntry { .. } => todo!(),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn text_input(&self) -> Option<&String> {
        if let Some(page) = self.current_page() {
            match page {
                Page::AddressBook { search_string, .. } => Some(search_string),
                Page::AddressBookCreateNewEntry {
                    cursor,
                    name,
                    address,
                    ..
                } => match cursor {
                    0 => Some(name),
                    1 => Some(address),
                    _ => None,
                },
                Page::AddressBookDisplayEntry { .. } => todo!(),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn up(&mut self) {
        if let Some(page) = self.current_page_mut() {
            match page {
                Page::MainMenu { list, cursor, .. } => {
                    let cursor_max = list.len();
                    *cursor = (*cursor + cursor_max - 1) % cursor_max;
                }
                Page::AddressBook {
                    full_list,
                    cursor,
                    search_string,
                } => {
                    let cursor_max = if search_string.is_empty() {
                        full_list.len()
                    } else {
                        full_list
                            .iter()
                            .filter(|entry| format!("{entry}").contains(search_string.as_str()))
                            .count()
                    };
                    *cursor = (*cursor + cursor_max - 1) % cursor_max;
                }
                Page::AddressBookCreateNewEntry { cursor, .. } => {
                    let cursor_max = 3;
                    *cursor = (*cursor + cursor_max - 1) % cursor_max;
                }
                _ => {}
            }
        }
    }

    pub fn down(&mut self) {
        if let Some(page) = self.current_page_mut() {
            match page {
                Page::MainMenu { list, cursor, .. } => {
                    let cursor_max = list.len();
                    *cursor = (*cursor + 1) % cursor_max;
                }
                Page::AddressBook {
                    full_list,
                    cursor,
                    search_string,
                } => {
                    let cursor_max = if search_string.is_empty() {
                        full_list.len()
                    } else {
                        full_list
                            .iter()
                            .filter(|entry| format!("{entry}").contains(search_string.as_str()))
                            .count()
                    };
                    *cursor = (*cursor + 1) % cursor_max;
                }
                Page::AddressBookCreateNewEntry { cursor, .. } => {
                    let cursor_max = 3;
                    *cursor = (*cursor + 1) % cursor_max;
                }
                _ => {}
            }
        }
    }

    pub fn enter(&mut self) {
        if let Some(current_page) = self.current_page_mut() {
            match current_page {
                Page::MainMenu { list, cursor, .. } => match &list[*cursor] {
                    Action::AddressBook { .. } => {
                        let full_list = AddressBookActions::get_menu();
                        self.pages.push(Page::AddressBook {
                            full_list,
                            search_string: String::new(),
                            cursor: 0,
                        });
                    }
                    _ => unimplemented!(),
                },
                Page::AddressBook {
                    full_list, cursor, ..
                } => {
                    let page = match &full_list[*cursor] {
                        AddressBookActions::Create { address, name } => {
                            Page::AddressBookCreateNewEntry {
                                cursor: 0,
                                name: name.clone().unwrap_or_default(),
                                address: address.map(|a| a.to_string()).unwrap_or_default(),
                                error: None,
                            }
                        }
                        AddressBookActions::View { id, address, name } => {
                            let (id, entry) = AddressBook::load()
                                .find(id, address, &name.as_ref())
                                .expect("entry not found");
                            Page::AddressBookDisplayEntry {
                                cursor: 0,
                                edit: false,
                                id,
                                name: entry.name,
                                address: entry.address,
                            }
                        }
                    };
                    self.pages.push(page);
                }
                Page::AddressBookCreateNewEntry {
                    cursor,
                    name,
                    address,
                    error,
                } => {
                    if *cursor == 2 {
                        if name.is_empty() {
                            *error =
                                Some("Please enter name, you cannot leave it empty".to_string());
                        } else {
                            let mut address_book = AddressBook::load();

                            let result =
                                address
                                    .parse()
                                    .map_err(crate::Error::from)
                                    .and_then(|address| {
                                        address_book.add(AddressBookEntry {
                                            name: name.clone(),
                                            address,
                                        })
                                    });
                            if let Err(e) = result {
                                *error = Some(format!("{e:?}"));
                            } else {
                                self.pages.pop();
                                self.pages.pop();
                                self.enter(); // trigger re-generation for the previous page
                            }
                        }
                    } else {
                        *cursor += 1;
                    }
                }
                _ => unimplemented!("{current_page:?}"),
            }
        } else {
            unreachable!()
        }
    }

    pub fn is_main_menu(&self) -> bool {
        self.pages.len() == 1
    }

    pub fn current_page(&self) -> Option<&Page> {
        self.pages.last()
    }

    pub fn is_text_input_active(&self) -> bool {
        self.text_input().is_some()
    }

    pub fn is_text_input_user_typing(&self) -> bool {
        self.text_input()
            .as_ref()
            .map(|s| s.len())
            .unwrap_or_default()
            != 0
    }

    pub fn current_page_mut(&mut self) -> Option<&mut Page> {
        self.pages.last_mut()
    }
}
