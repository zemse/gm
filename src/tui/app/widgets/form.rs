use std::{collections::HashMap, marker::PhantomData};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    style::Stylize,
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
};
use strum::IntoEnumIterator;

use crate::tui::{
    traits::{RectUtil, WidgetHeight},
    Event,
};

use super::{button::Button, input_box::InputBox};

pub trait FormItemIndex {
    fn index(self) -> usize;
}

#[derive(Clone, Debug)]
pub enum FormWidget {
    Heading(&'static str),
    StaticText(&'static str),
    InputBox {
        label: &'static str,
        text: String,
        empty_text: Option<&'static str>,
        currency: Option<String>,
    },
    BooleanInput {
        label: &'static str,
        value: bool,
    },
    DisplayBox {
        label: &'static str,
        text: String,
        empty_text: Option<&'static str>,
    },
    Button {
        label: &'static str,
    },
    DisplayText(String),
    ErrorText(String),
}

impl FormWidget {
    pub fn label(&self) -> Option<&'static str> {
        match self {
            FormWidget::InputBox { label, .. } => Some(label),
            FormWidget::DisplayBox { label, .. } => Some(label),
            FormWidget::BooleanInput { label, .. } => Some(label),
            FormWidget::Button { label } => Some(label),
            FormWidget::Heading(_)
            | FormWidget::StaticText(_)
            | FormWidget::DisplayText(_)
            | FormWidget::ErrorText(_) => None,
        }
    }

    pub fn max_cursor(&self) -> usize {
        match self {
            FormWidget::InputBox { text, .. } => text.len(),
            FormWidget::DisplayBox { text, .. } => text.len(),
            FormWidget::Button { .. }
            | FormWidget::BooleanInput { .. }
            | FormWidget::Heading(_)
            | FormWidget::StaticText(_)
            | FormWidget::DisplayText(_)
            | FormWidget::ErrorText(_) => 0,
        }
    }
}

pub struct Form<E: IntoEnumIterator + FormItemIndex + TryInto<FormWidget, Error = crate::Error>> {
    pub cursor: usize,
    pub text_cursor: usize,
    pub form_focus: bool,
    pub items: Vec<FormWidget>,
    pub hide: HashMap<usize, bool>,
    pub everything_empty: bool,
    pub _phantom: PhantomData<E>,
}

impl<E: IntoEnumIterator + FormItemIndex + TryInto<FormWidget, Error = crate::Error>> Form<E> {
    // TODO remove the cursor parameter, and guess it as the first item that is
    // not heading or static text or similar
    pub fn init<F>(set_values_closure: F) -> crate::Result<Self>
    where
        F: FnOnce(&mut Self) -> crate::Result<()>,
    {
        let mut form = Self {
            cursor: 0,
            text_cursor: 0,
            form_focus: true,
            items: E::iter()
                .map(|item| item.try_into())
                .collect::<Result<Vec<FormWidget>, _>>()?,
            hide: HashMap::new(),
            everything_empty: false,
            _phantom: PhantomData,
        };
        for i in 0..form.items.len() {
            if form.is_valid_cursor(i) {
                break;
            } else {
                form.cursor += 1;
            }
        }
        set_values_closure(&mut form)?;
        form.text_cursor = form.items[form.cursor].max_cursor();

        Ok(form)
    }

    pub fn set_form_focus(&mut self, focus: bool) {
        self.form_focus = focus;
    }

    pub fn show_everything_empty(&mut self, empty: bool) {
        self.everything_empty = empty;
    }

    pub fn hide_item(&mut self, idx: E) {
        self.hide.insert(idx.index(), true);
    }

    pub fn show_item(&mut self, idx: E) {
        self.hide.remove(&idx.index());
    }

    pub fn hidden_count(&self) -> usize {
        self.hide.len()
    }

    pub fn visible_count(&self) -> usize {
        self.items.len() - self.hidden_count()
    }

    pub fn advance_cursor(&mut self) {
        loop {
            self.cursor = (self.cursor + 1) % self.items.len();
            self.text_cursor = self.items[self.cursor].max_cursor();

            if self.is_valid_cursor(self.cursor) {
                break;
            }
        }
    }

    pub fn retreat_cursor(&mut self) {
        loop {
            self.cursor = (self.cursor + self.items.len() - 1) % self.items.len();
            self.text_cursor = self.items[self.cursor].max_cursor();

            if self.is_valid_cursor(self.cursor) {
                break;
            }
        }
    }

    pub fn is_valid_cursor(&self, idx: usize) -> bool {
        if self.hide.contains_key(&idx) {
            return false;
        }

        match &self.items[idx] {
            FormWidget::Heading(_)
            | FormWidget::StaticText(_)
            | FormWidget::DisplayText(_)
            | FormWidget::ErrorText(_) => false,

            FormWidget::InputBox { .. }
            | FormWidget::DisplayBox { .. }
            | FormWidget::BooleanInput { .. }
            | FormWidget::Button { .. } => true,
        }
    }

    pub fn get_text(&self, idx: E) -> &String {
        match &self.items[idx.index()] {
            FormWidget::InputBox { text, .. } => text,
            FormWidget::DisplayBox { text, .. } => text,
            FormWidget::DisplayText(text) => text,
            FormWidget::ErrorText(text) => text,
            _ => unreachable!(),
        }
    }

    pub fn get_text_mut(&mut self, idx: E) -> &mut String {
        match &mut self.items[idx.index()] {
            FormWidget::InputBox { text, .. } => text,
            FormWidget::DisplayBox { text, .. } => text,
            FormWidget::DisplayText(text) => text,
            FormWidget::ErrorText(text) => text,
            _ => unreachable!(),
        }
    }

    pub fn get_boolean(&self, idx: E) -> bool {
        match &self.items[idx.index()] {
            FormWidget::BooleanInput { value, .. } => *value,
            _ => unreachable!(),
        }
    }

    pub fn get_boolean_mut(&mut self, idx: E) -> &mut bool {
        match &mut self.items[idx.index()] {
            FormWidget::BooleanInput { value, .. } => value,
            _ => unreachable!(),
        }
    }

    pub fn get_currency_mut(&mut self, idx: E) -> Option<&mut Option<String>> {
        match &mut self.items[idx.index()] {
            FormWidget::InputBox { currency, .. } => Some(currency),
            _ => None,
        }
    }

    pub fn is_focused(&self, idx: E) -> bool {
        self.cursor == idx.index()
    }

    pub fn is_button_focused(&self) -> bool {
        matches!(self.items[self.cursor], FormWidget::Button { .. })
    }

    pub fn handle_event<F>(&mut self, event: &Event, mut on_button: F) -> crate::Result<()>
    where
        F: FnMut(E, &mut Self) -> crate::Result<()>,
    {
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Up => loop {
                        self.retreat_cursor();

                        if !self.hide.contains_key(&self.cursor) {
                            match &self.items[self.cursor] {
                                FormWidget::InputBox { .. }
                                | FormWidget::DisplayBox { .. }
                                | FormWidget::BooleanInput { .. }
                                | FormWidget::Button { .. } => break,
                                _ => {}
                            }
                        }
                    },
                    KeyCode::Down | KeyCode::Tab => loop {
                        self.advance_cursor();

                        if self.is_valid_cursor(self.cursor) {
                            break;
                        }
                    },
                    KeyCode::Enter => {
                        if !self.is_button_focused() {
                            loop {
                                self.cursor = (self.cursor + 1) % self.items.len();

                                if self.is_valid_cursor(self.cursor) {
                                    break;
                                }
                            }
                        }
                    }

                    _ => {}
                }

                match &mut self.items[self.cursor] {
                    FormWidget::InputBox { text, .. } => {
                        InputBox::handle_events(text, &mut self.text_cursor, event)?;
                    }
                    FormWidget::DisplayBox { .. } => {
                        // we don't have to handle this as parent component will do it
                    }
                    FormWidget::BooleanInput { value, .. } => {
                        if matches!(
                            key_event.code,
                            KeyCode::Char(_) | KeyCode::Left | KeyCode::Right | KeyCode::Backspace
                        ) {
                            *value = !*value
                        }
                    }
                    FormWidget::Button { .. } => {
                        if matches!(key_event.code, KeyCode::Enter) {
                            let enum_repr =
                                E::iter().nth(self.cursor).expect("Invalid cursor index");
                            on_button(enum_repr, self)?
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl<E: IntoEnumIterator + FormItemIndex + TryInto<FormWidget, Error = crate::Error>> Widget
    for &Form<E>
{
    fn render(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        for (i, item) in self.items.iter().enumerate() {
            if self.hide.contains_key(&i) {
                continue; // skip hidden items
            }

            match item {
                FormWidget::Heading(heading) => {
                    heading.bold().render(area, buf);
                    area.y += 2;
                }
                FormWidget::StaticText(text) => {
                    text.render(area, buf);
                    area.y += 2;
                }
                FormWidget::InputBox {
                    label,
                    text,
                    empty_text,
                    currency,
                } => {
                    let widget = InputBox {
                        focus: self.form_focus && self.cursor == i,
                        label,
                        text: if !self.everything_empty {
                            text
                        } else {
                            &"".to_string()
                        },
                        empty_text: if !self.everything_empty {
                            *empty_text
                        } else {
                            Some("")
                        },
                        currency: currency.as_ref(),
                    };
                    let height_used = widget.height_used(area); // to see height based on width
                    widget.render(area, buf, &self.text_cursor);
                    area.y += height_used;
                }
                FormWidget::DisplayBox {
                    label,
                    text,
                    empty_text,
                } => {
                    let widget = InputBox {
                        focus: self.form_focus && self.cursor == i,
                        label,
                        text: if !self.everything_empty {
                            text
                        } else {
                            &"".to_string()
                        },
                        empty_text: if !self.everything_empty {
                            *empty_text
                        } else {
                            Some("")
                        },
                        currency: None,
                    };
                    let height_used = widget.height_used(area); // to see height based on width
                    widget.render(area, buf, &self.text_cursor);
                    area.y += height_used;
                }
                FormWidget::BooleanInput { label, value } => {
                    let widget = InputBox {
                        focus: self.form_focus && self.cursor == i,
                        label,
                        text: if !self.everything_empty {
                            &value.to_string()
                        } else {
                            &"".to_string()
                        },
                        empty_text: None,
                        currency: None,
                    };
                    let height_used = widget.height_used(area); // to see height based on width
                    widget.render(area, buf, &self.text_cursor);
                    area.y += height_used;
                }
                FormWidget::Button { label } => {
                    Button {
                        focus: self.form_focus && self.cursor == i,
                        label,
                    }
                    .render(area, buf);
                    area.y += 4;
                }
                FormWidget::DisplayText(text) | FormWidget::ErrorText(text) => {
                    if !text.is_empty() {
                        area.y += 1;
                        Paragraph::new(Text::raw(text))
                            .wrap(Wrap { trim: false })
                            .render(area.margin_h(1), buf);
                        area.y += (text.len() as u16).div_ceil(area.width) + 1;
                    }
                }
            }
        }
    }
}
