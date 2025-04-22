use std::sync::{atomic::AtomicBool, mpsc, Arc};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::Widget;

use crate::{
    actions::address_book::AddressBookActions,
    disk::{AddressBook, DiskInterface},
    tui::{
        app::widgets::filter_select::FilterSelect,
        events::Event,
        traits::{Component, HandleResult},
    },
};

use super::{
    address_book_create::AddressBookCreatePage, address_book_display::AddressBookDisplayPage, Page,
};

pub struct AddressBookPage {
    full_list: Vec<AddressBookActions>,
    search_string: String,
    cursor: usize,
}

impl Default for AddressBookPage {
    fn default() -> Self {
        Self {
            full_list: AddressBookActions::get_menu(),
            search_string: String::new(),
            cursor: 0,
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
    ) -> crate::Result<HandleResult> {
        let list: Vec<&AddressBookActions> = self
            .full_list
            .iter()
            .filter(|entry| format!("{entry}").contains(self.search_string.as_str()))
            .collect();

        let cursor_max = list.len();

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
                    KeyCode::Up => {
                        self.cursor = (self.cursor + cursor_max - 1) % cursor_max;
                    }
                    KeyCode::Down => {
                        self.cursor = (self.cursor + 1) % cursor_max;
                    }
                    KeyCode::Enter => result.page_inserts.push(match &list[self.cursor] {
                        AddressBookActions::Create { address, name } => {
                            Page::AddressBookCreate(AddressBookCreatePage {
                                cursor: 0,
                                name: name.clone().unwrap_or_default(),
                                address: address.map(|a| a.to_string()).unwrap_or_default(),
                                error: None,
                            })
                        }
                        AddressBookActions::View { id, address, name } => {
                            let (id, entry) = AddressBook::load()
                                .find(id, address, &name.as_ref())
                                .expect("entry not found");
                            Page::AddressBookDisplay(AddressBookDisplayPage {
                                cursor: 0,
                                id,
                                name: entry.name,
                                address: entry.address.to_string(),
                                error: None,
                            })
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
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        FilterSelect {
            full_list: &self.full_list,
            cursor: Some(&self.cursor),
            search_string: &self.search_string,
        }
        .render(area, buf);
        area
    }
}
