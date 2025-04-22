use std::sync::{atomic::AtomicBool, mpsc, Arc};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::Widget;

use crate::{
    disk::{AddressBook, AddressBookEntry, DiskInterface},
    tui::{
        app::widgets::form::{Form, FormItem},
        events::Event,
        traits::{Component, HandleResult},
    },
};
pub struct AddressBookCreatePage {
    pub cursor: usize,
    pub name: String,
    pub address: String,
    pub error: Option<String>,
}

impl Component for AddressBookCreatePage {
    fn text_input_mut(&mut self) -> Option<&mut String> {
        match self.cursor {
            0 => Some(&mut self.name),
            1 => Some(&mut self.address),
            _ => None,
        }
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
    ) -> crate::Result<HandleResult> {
        let cursor_max = 2;

        let mut handle_result = HandleResult::default();
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
                    KeyCode::Enter => {
                        if self.cursor == 2 {
                            if self.name.is_empty() {
                                self.error = Some(
                                    "Please enter name, you cannot leave it empty".to_string(),
                                );
                            } else {
                                let mut address_book = AddressBook::load();

                                let result =
                                    self.address.parse().map_err(crate::Error::from).and_then(
                                        |address| {
                                            address_book.add(AddressBookEntry {
                                                name: self.name.clone(),
                                                address,
                                            })
                                        },
                                    );
                                if let Err(e) = result {
                                    self.error = Some(format!("{e:?}"));
                                } else {
                                    handle_result.page_pops = 1;
                                    handle_result.reload = true;
                                }
                            }
                        } else {
                            // TODO handle overflow on cursor_max
                            self.cursor += 1;
                        }
                    }
                    KeyCode::Tab => self.cursor += 1,
                    _ => {}
                }
            }
        }

        Ok(handle_result)
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        Form {
            items: vec![
                FormItem::Heading("Edit AddressBook entry"),
                FormItem::InputBox {
                    focus: self.cursor == 0,
                    label: &"name".to_string(),
                    text: &self.name,
                },
                FormItem::InputBox {
                    focus: self.cursor == 1,
                    label: &"address".to_string(),
                    text: &self.address,
                },
                FormItem::Button {
                    focus: self.cursor == 2,
                    label: &"Save".to_string(),
                },
                FormItem::Error {
                    label: &self.error.as_ref(),
                },
            ],
        }
        .render(area, buf);

        area
    }
}
