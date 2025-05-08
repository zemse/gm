use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{style::Stylize, widgets::Widget};

use crate::tui::{traits::WidgetHeight, Event};

use super::{button::Button, input_box::InputBox};

pub enum FormItem {
    Heading(&'static str),
    InputBox {
        label: &'static str,
        text: String,
        empty_text: Option<&'static str>,
    },
    BooleanInput {
        label: &'static str,
        value: bool,
    },
    Button {
        label: &'static str,
    },
    DisplayText(String),
    ErrorText(String),
}

impl FormItem {
    pub fn label(&self) -> Option<&'static str> {
        match self {
            FormItem::InputBox { label, .. } => Some(label),
            FormItem::BooleanInput { label, .. } => Some(label),
            FormItem::Button { label } => Some(label),
            _ => None,
        }
    }
}

pub struct Form {
    pub cursor: usize,
    pub items: Vec<FormItem>,
}

impl Form {
    pub fn get_input_text(&self, idx: usize) -> &String {
        match &self.items[idx] {
            FormItem::InputBox { text, .. } => text,
            _ => unreachable!(),
        }
    }

    pub fn get_input_text_mut(&mut self, idx: usize) -> &mut String {
        match &mut self.items[idx] {
            FormItem::InputBox { text, .. } => text,
            _ => unreachable!(),
        }
    }

    pub fn get_boolean_value(&self, idx: usize) -> bool {
        match &self.items[idx] {
            FormItem::BooleanInput { value, .. } => *value,
            _ => unreachable!(),
        }
    }

    pub fn get_display_text_mut(&mut self, idx: usize) -> &mut String {
        match &mut self.items[idx] {
            FormItem::DisplayText(text, ..) => text,
            _ => unreachable!(),
        }
    }

    pub fn get_error_text_mut(&mut self, idx: usize) -> &mut String {
        match &mut self.items[idx] {
            FormItem::ErrorText(text, ..) => text,
            _ => unreachable!(),
        }
    }

    pub fn is_focused(&self, label: &str) -> bool {
        self.items[self.cursor].label() == Some(label)
    }

    pub fn is_button_focused(&self) -> bool {
        matches!(self.items[self.cursor], FormItem::Button { .. })
    }

    pub fn handle_event<F>(&mut self, event: &Event, mut on_button: F) -> crate::Result<()>
    where
        F: FnMut(&'static str, &mut Self),
    {
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Up => loop {
                        self.cursor = (self.cursor + self.items.len() - 1) % self.items.len();

                        match &self.items[self.cursor] {
                            FormItem::InputBox { .. } => break,
                            FormItem::BooleanInput { .. } => break,
                            FormItem::Button { .. } => break,
                            _ => {}
                        }
                    },
                    KeyCode::Down | KeyCode::Tab => loop {
                        self.cursor = (self.cursor + 1) % self.items.len();

                        match &self.items[self.cursor] {
                            FormItem::InputBox { .. } => break,
                            FormItem::BooleanInput { .. } => break,
                            FormItem::Button { .. } => break,
                            _ => {}
                        }
                    },
                    KeyCode::Enter => {
                        if !self.is_button_focused() {
                            loop {
                                self.cursor = (self.cursor + 1) % self.items.len();

                                match &self.items[self.cursor] {
                                    FormItem::InputBox { .. } => break,
                                    FormItem::BooleanInput { .. } => break,
                                    FormItem::Button { .. } => break,
                                    _ => {}
                                }
                            }
                        }
                    }

                    _ => {}
                }

                match &mut self.items[self.cursor] {
                    FormItem::InputBox { text, .. } => {
                        InputBox::handle_events(text, event)?;
                    }
                    FormItem::BooleanInput { value, .. } => {
                        if matches!(
                            key_event.code,
                            KeyCode::Char(_) | KeyCode::Left | KeyCode::Right | KeyCode::Backspace
                        ) {
                            *value = !*value
                        }
                    }
                    FormItem::Button { label } => {
                        if matches!(key_event.code, KeyCode::Enter) {
                            on_button(label, self)
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Widget for &Form {
    fn render(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        for (i, item) in self.items.iter().enumerate() {
            match item {
                FormItem::Heading(heading) => {
                    heading.bold().render(area, buf);
                    area.y += 2;
                }
                FormItem::InputBox {
                    label,
                    text,
                    empty_text,
                } => {
                    let widget = InputBox {
                        focus: self.cursor == i,
                        label,
                        text,
                        empty_text: *empty_text,
                    };
                    let height_used = widget.height_used(area); // to see height based on width
                    widget.render(area, buf);
                    area.y += height_used;
                }
                FormItem::BooleanInput { label, value } => {
                    let widget = InputBox {
                        focus: self.cursor == i,
                        label,
                        text: &value.to_string(),
                        empty_text: None,
                    };
                    let height_used = widget.height_used(area); // to see height based on width
                    widget.render(area, buf);
                    area.y += height_used;
                }
                FormItem::Button { label } => {
                    Button {
                        focus: self.cursor == i,
                        label,
                    }
                    .render(area, buf);
                    area.y += 3;
                }
                FormItem::DisplayText(text) | FormItem::ErrorText(text) => {
                    if !text.is_empty() {
                        area.y += 1;
                        text.render(area, buf);
                        area.y += 1;
                    }
                }
            }
        }
    }
}
