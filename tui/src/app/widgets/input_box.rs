use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Offset, Rect},
    text::Span,
    widgets::{Block, Widget},
};

use crate::theme::Theme;
use crate::{
    traits::{HandleResult, WidgetHeight},
    Event,
};
use gm_utils::text::split_string;

fn option_delete(text_input: &mut String, text_cursor: &mut usize) {
    loop {
        if *text_cursor == 0 {
            break;
        }
        text_input.remove(*text_cursor - 1);
        *text_cursor -= 1;
        if *text_cursor == 0 {
            break;
        }
        let next_char = text_input.chars().nth(*text_cursor - 1).unwrap_or(' ');
        if next_char == ' ' {
            break;
        }
    }
}

fn option_left(text_input: &str, text_cursor: &mut usize) {
    loop {
        if *text_cursor == 0 {
            break;
        }
        *text_cursor -= 1;
        let cur_char = text_input.chars().nth(*text_cursor).unwrap_or(' ');
        if cur_char == ' ' {
            break;
        }
    }
}

fn option_right(text_input: &str, text_cursor: &mut usize) {
    loop {
        if *text_cursor == text_input.len() {
            break;
        }
        *text_cursor += 1;
        let cur_char = text_input.chars().nth(*text_cursor).unwrap_or(' ');
        if cur_char == ' ' {
            break;
        }
    }
}

pub struct InputBox<'a> {
    pub focus: bool,
    pub label: &'static str,
    pub text: &'a String,
    pub empty_text: Option<&'static str>,
    pub currency: Option<&'a String>,
}

impl InputBox<'_> {
    pub fn handle_events(
        text_input: &mut String,
        text_cursor: &mut usize,
        event: &Event,
    ) -> crate::Result<HandleResult> {
        let result = HandleResult::default();

        if let Event::Input(key_event) = event {
            match key_event.code {
                KeyCode::Left => {
                    if key_event.modifiers == KeyModifiers::ALT {
                        option_left(text_input, text_cursor);
                    } else if *text_cursor > 0 {
                        *text_cursor -= 1
                    }
                }
                KeyCode::Right => {
                    if key_event.modifiers == KeyModifiers::ALT {
                        option_right(text_input, text_cursor);
                    } else if *text_cursor < text_input.len() {
                        *text_cursor += 1
                    }
                }
                KeyCode::Char(char) => {
                    // Handle space key on empty state
                    if text_input.is_empty() && char == ' ' {
                        // Ignore leading spaces
                    }
                    // Handle command + delete on macOS
                    else if char == 'u' && key_event.modifiers == KeyModifiers::CONTROL {
                        let (_, right) = text_input.split_at(*text_cursor);
                        *text_input = right.to_string();
                        *text_cursor = 0;
                    }
                    // Handle command + left on macOS
                    else if char == 'a' && key_event.modifiers == KeyModifiers::CONTROL {
                        *text_cursor = 0;
                    }
                    // Handle command + right on macOS
                    else if char == 'e' && key_event.modifiers == KeyModifiers::CONTROL {
                        *text_cursor = text_input.len();
                    }
                    // Handle option + delete on macOS
                    else if char == 'w' && key_event.modifiers == KeyModifiers::CONTROL {
                        option_delete(text_input, text_cursor);
                    }
                    // option + Left
                    else if char == 'b' && key_event.modifiers == KeyModifiers::ALT {
                        option_left(text_input, text_cursor);
                    }
                    // option + Right
                    else if char == 'f' && key_event.modifiers == KeyModifiers::ALT {
                        option_right(text_input, text_cursor);
                    }
                    // Simple char press
                    else {
                        text_input.insert(*text_cursor, char);
                        *text_cursor += 1;
                    }
                }
                KeyCode::Backspace => {
                    if key_event.modifiers == KeyModifiers::ALT {
                        option_delete(text_input, text_cursor);
                    } else if *text_cursor > 0 {
                        *text_cursor -= 1;
                        text_input.remove(*text_cursor);
                    }
                }
                _ => {}
            }
        }

        // *text_input = event.fmt();

        Ok(result)
    }

    pub fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        text_cursor: &usize,
        theme: &Theme,
    ) where
        Self: Sized,
    {
        let lines = split_string(self.text, (area.width - 2) as usize);
        let area_used = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: (2 + lines.len()) as u16,
        };

        let block = Block::bordered()
            .border_type(theme.into())
            .title(self.label);
        let inner_area = block.inner(area_used);
        block.render(area_used, buf);

        if lines.len() == 1 && !lines.last().unwrap().is_empty() && self.currency.is_some() {
            let currency = self.currency.unwrap();
            Span::from(currency).render(
                inner_area.offset(Offset {
                    x: lines.last().unwrap().len() as i32 + 1,
                    y: 0,
                }),
                buf,
            );
        }

        for (idx, line) in lines.iter().enumerate() {
            line.render(
                inner_area.offset(Offset {
                    x: 0,
                    y: idx as i32,
                }),
                buf,
            );
        }

        if self.focus {
            if self.text.is_empty() && self.empty_text.is_some() {
                self.empty_text.unwrap().render(inner_area, buf);
            } else {
                Span::from("|").render(
                    Rect {
                        x: inner_area.x + (*text_cursor as u16) % (area.width - 2),
                        y: inner_area.y + (*text_cursor as u16) / (area.width - 2),
                        width: 1,
                        height: 1,
                    },
                    buf,
                );
            }
        }
    }
}

impl WidgetHeight for InputBox<'_> {
    fn height_used(&self, area: ratatui::prelude::Rect) -> u16 {
        let lines = split_string(self.text, (area.width - 2) as usize);
        (2 + lines.len()) as u16
    }
}
