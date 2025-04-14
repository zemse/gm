use crate::actions::{address_book::AddressBookActions, Action};

pub enum Page {
    MainMenu {
        list: Vec<Action>,
        cursor: usize,
    },
    AddressBook {
        full_list: Vec<AddressBookActions>,
        cursor: usize,
        search_string: String,
    },
    Input {
        prompt: String,
        input: String,
    },
}
pub struct Navigation {
    pages: Vec<Page>,
}

impl Default for Navigation {
    fn default() -> Self {
        let list = Action::get_menu();
        Self {
            pages: vec![Page::MainMenu {
                // cursor_max: list.len(),
                list,
                cursor: 0,
            }],
        }
    }
}

impl Navigation {
    pub fn current_page(&self) -> &Page {
        self.pages.last().unwrap()
    }

    pub fn current_page_mut(&mut self) -> &mut Page {
        self.pages.last_mut().unwrap()
    }

    pub fn up(&mut self) {
        match self.current_page_mut() {
            Page::MainMenu { list, cursor, .. } => {
                let cursor_max = list.len();
                *cursor = (*cursor + cursor_max - 1) % cursor_max;
            }
            Page::AddressBook {
                full_list,
                cursor,
                search_string,
            } => {
                let cursor_max = full_list
                    .iter()
                    .filter(|entry| format!("{entry}").contains(search_string.as_str()))
                    .count();
                *cursor = (*cursor + cursor_max - 1) % cursor_max;
            }
            _ => {}
        }
    }

    pub fn down(&mut self) {
        match self.current_page_mut() {
            Page::MainMenu { list, cursor, .. } => {
                let cursor_max = list.len();
                *cursor = (*cursor + 1) % cursor_max;
            }
            Page::AddressBook {
                full_list,
                cursor,
                search_string,
            } => {
                let cursor_max = full_list
                    .iter()
                    .filter(|entry| format!("{entry}").contains(search_string.as_str()))
                    .count();
                *cursor = (*cursor + 1) % cursor_max;
            }
            _ => {}
        }
    }

    pub fn enter(&mut self) {
        match self.current_page() {
            Page::MainMenu { list, cursor, .. } => match &list[*cursor] {
                Action::AddressBook { .. } => {
                    let full_list = AddressBookActions::get_menu();
                    self.pages.push(Page::AddressBook {
                        full_list,
                        cursor: 0,
                        search_string: String::new(),
                    });
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}
