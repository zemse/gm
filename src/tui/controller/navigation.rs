use std::marker::PhantomData;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::actions::{address_book::AddressBookActions, Action};

pub enum Page {
    MainMenu {
        list: Vec<Action>,
        cursor: usize,
    },
    AddressBook {
        full_list: Vec<AddressBookActions>,
        cursor: usize,
    },
    Input {
        prompt: String,
        input: String,
    },
}
pub struct Navigation<'a> {
    pages: Vec<Page>,
    pub text_input: Option<String>,
    _marker: PhantomData<&'a ()>,
}

impl Default for Navigation<'_> {
    fn default() -> Self {
        let list = Action::get_menu();
        Self {
            pages: vec![Page::MainMenu { list, cursor: 0 }],
            text_input: None,
            _marker: PhantomData,
        }
    }
}

impl Navigation<'_> {
    pub fn handle(&mut self, key_event: KeyEvent) {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char(char) => {
                    if let Some(text_input) = self.text_input.as_mut() {
                        text_input.push(char);
                    }
                }
                KeyCode::Backspace => {
                    if let Some(text_input) = self.text_input.as_mut() {
                        text_input.pop();
                    }
                }
                // KeyCode::Esc => {
                //     if !self.navigation.esc() {
                //         self.exit = true;
                //     }
                // }
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

    pub fn current_page(&self) -> &Page {
        self.pages.last().unwrap()
    }

    pub fn enable_text_input(&mut self) {
        self.text_input = Some(String::new());
    }

    pub fn disable_text_input(&mut self) {
        self.text_input = None;
    }

    pub fn current_page_mut(&mut self) -> &mut Page {
        self.pages.last_mut().unwrap()
    }

    pub fn up(&mut self) {
        let search_string = self.text_input.clone();
        match self.current_page_mut() {
            Page::MainMenu { list, cursor, .. } => {
                let cursor_max = list.len();
                *cursor = (*cursor + cursor_max - 1) % cursor_max;
            }
            Page::AddressBook { full_list, cursor } => {
                let cursor_max = if let Some(search_string) = search_string {
                    full_list
                        .iter()
                        .filter(|entry| format!("{entry}").contains(search_string.as_str()))
                        .count()
                } else {
                    full_list.len()
                };
                *cursor = (*cursor + cursor_max - 1) % cursor_max;
            }
            _ => {}
        }
    }

    pub fn down(&mut self) {
        let search_string = self.text_input.clone();
        match self.current_page_mut() {
            Page::MainMenu { list, cursor, .. } => {
                let cursor_max = list.len();
                *cursor = (*cursor + 1) % cursor_max;
            }
            Page::AddressBook { full_list, cursor } => {
                let cursor_max = if let Some(search_string) = search_string {
                    full_list
                        .iter()
                        .filter(|entry| format!("{entry}").contains(search_string.as_str()))
                        .count()
                } else {
                    full_list.len()
                };
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
                    });
                    self.text_input = Some(String::new());
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}
