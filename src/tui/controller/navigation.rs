use crate::actions::Action;

pub enum Page {
    MainMenu {
        list: Vec<Action>,
        cursor: usize,
        cursor_max: usize,
    },
    AddressBook {
        full_list: Vec<String>,
        list: Vec<String>,
        cursor: usize,
        search_string: String,
        cursor_max: usize,
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
                cursor_max: list.len(),
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
            Page::MainMenu {
                cursor_max, cursor, ..
            }
            | Page::AddressBook {
                cursor_max, cursor, ..
            } => {
                *cursor = (*cursor + *cursor_max) % (*cursor_max + 1);
            }
            _ => {}
        }
    }

    pub fn down(&mut self) {
        match self.current_page_mut() {
            Page::MainMenu {
                cursor_max, cursor, ..
            }
            | Page::AddressBook {
                cursor_max, cursor, ..
            } => {
                *cursor = (*cursor + 1) % (*cursor_max + 1);
            }
            _ => {}
        }
    }
}
