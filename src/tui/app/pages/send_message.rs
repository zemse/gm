use crate::disk::{AddressBook, AddressBookEntry, DiskInterface};
use crate::tui::app::widgets::filter_select::FilterSelect;
use crate::tui::app::widgets::popup::Popup;
use crate::tui::app::SharedState;
use crate::tui::{
    app::widgets::form::{Form, FormItem}, // <- Using your custom form system
    events::Event,
    traits::{Component, HandleResult},
};
use crate::utils::cursor::Cursor;
use crate::Result;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::style::Color;
use ratatui::widgets::{Block, Widget};
use std::sync::mpsc;
use std::sync::{atomic::AtomicBool, Arc};

pub struct SendMessagePage {
    pub form: Form,
    /// Address book popup state
    pub address_book: Option<AddressBook>,
    pub cursor: Cursor,
    pub search_string: String,
}

const TO: &str = "To";
const MESSAGE: &str = "Message";
const SEND_MESSAGE: &str = "Send Message";

impl Default for SendMessagePage {
    fn default() -> Self {
        Self {
            form: Form {
                cursor: 1,
                items: vec![
                    FormItem::Heading("Send a Message"),
                    FormItem::InputBox {
                        label: TO,
                        text: String::new(),
                        empty_text: Some("<press SPACE to select from address book>"),
                    },
                    FormItem::InputBox {
                        label: MESSAGE,
                        text: String::new(),
                        empty_text: None,
                    },
                    FormItem::Button {
                        label: SEND_MESSAGE,
                    },
                ],
            },
            address_book: None,
            cursor: Cursor::default(),
            search_string: String::new(),
        }
    }
}

impl Component for SendMessagePage {
    fn handle_event(
        &mut self,
        event: &Event,
        _tr: &mpsc::Sender<Event>,
        _sd: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> Result<HandleResult> {
        let mut result = HandleResult::default();

        // Keyboard events focus on the form is there is no address book popup
        if self.address_book.is_none() {
            self.form.handle_event(event, |_label, _form| {})?;
        } else {
            // TODO refactor this code into FilterSelect module
            let list: Vec<&AddressBookEntry> = self
                .address_book
                .as_ref()
                .unwrap()
                .list()
                .iter()
                .filter(|entry| format!("{entry}").contains(self.search_string.as_str()))
                .collect();

            let cursor_max = list.len();
            self.cursor.handle(event, cursor_max);

            if let Event::Input(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char(char) => {
                            self.search_string.push(char);
                        }
                        KeyCode::Backspace => {
                            self.search_string.pop();
                        }
                        KeyCode::Enter => {
                            let to_address = self.form.get_input_text_mut(1);
                            *to_address = list[self.cursor.current].address.to_string();
                            self.address_book = None;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Activate the address book popup if the user presses SPACE in the "To" field
        if self.form.is_focused(TO)
            && self.form.get_input_text(1).is_empty()
            && event.is_char_pressed(Some(' '))
        {
            let ab = AddressBook::load();
            self.address_book = Some(ab);
        }

        if self.address_book.is_some() {
            result.esc_ignores = 1;
        }

        if event.is_key_pressed(KeyCode::Esc) {
            self.address_book = None;
        }

        Ok(result)
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        _: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf);

        if let Some(address_book) = &self.address_book {
            Popup {
                bg_color: Some(Color::Blue),
            }
            .render(area, buf);

            let inner_area = Popup::inner_area(area);
            let block = Block::bordered().title("Address Book");
            let block_inner_area = block.inner(inner_area);
            block.render(inner_area, buf);

            FilterSelect {
                full_list: address_book.list(),
                cursor: &self.cursor,
                search_string: &self.search_string,
            }
            .render(block_inner_area, buf);
        }

        area
    }
}
