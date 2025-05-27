use crate::disk::{AddressBook, AddressBookEntry, DiskInterface};
use crate::tui::app::widgets::filter_select::FilterSelect;
use crate::tui::app::widgets::form::FormItemIndex;
use crate::tui::app::widgets::popup::Popup;
use crate::tui::app::{Focus, SharedState};
use crate::tui::{
    app::widgets::form::{Form, FormWidget}, // <- Using your custom form system
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
use strum::EnumIter;

#[derive(EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    To,
    Message,
    SendMessageButton,
}
impl FormItemIndex for FormItem {
    fn index(self) -> usize {
        self as usize
    }
}
impl From<FormItem> for FormWidget {
    fn from(value: FormItem) -> Self {
        match value {
            FormItem::Heading => FormWidget::Heading("Send a Message"),
            FormItem::To => FormWidget::InputBox {
                label: "To",
                text: String::new(),
                empty_text: Some("<press SPACE to select from address book>"),
                currency: None,
            },
            FormItem::Message => FormWidget::InputBox {
                label: "Message",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::SendMessageButton => FormWidget::Button {
                label: "Send Message",
            },
        }
    }
}

pub struct SendMessagePage {
    pub form: Form<FormItem>,
    /// Address book popup state
    pub address_book: Option<AddressBook>,
    pub cursor: Cursor,
    pub search_string: String,
}

impl Default for SendMessagePage {
    fn default() -> Self {
        Self {
            form: Form::init(1),
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
            self.form.handle_event(event, |_label, _form| Ok(()))?;
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
                            let to_address = self.form.get_input_text_mut(FormItem::To);
                            *to_address = list[self.cursor.current].address.to_string();
                            self.address_book = None;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Activate the address book popup if the user presses SPACE in the "To" field
        if self.form.is_focused(FormItem::To)
            && self.form.get_input_text(FormItem::To).is_empty()
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
        shared_state: &SharedState,
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
                focus: shared_state.focus == Focus::Main,
            }
            .render(block_inner_area, buf);
        }

        area
    }
}
