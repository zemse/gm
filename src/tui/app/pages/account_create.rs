use std::sync::mpsc;

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Widget,
};

use crate::tui::{
    events::Event,
    traits::{Component, HandleResult},
};

pub struct AccountCreatePage {
    pub cursor: usize,
    pub mask: [Option<u8>; 40],
    pub error: Option<String>,
}

impl Default for AccountCreatePage {
    fn default() -> Self {
        Self {
            cursor: 0,
            mask: [None; 40],
            error: None,
        }
    }
}

impl Component for AccountCreatePage {
    fn handle_event(
        &mut self,
        event: &Event,
        _transmitter: &mpsc::Sender<Event>,
    ) -> crate::Result<HandleResult> {
        let result = HandleResult::default();

        if let Event::Input(key_event) = event {
            let cursor_max = self.mask.len();

            match key_event.code {
                KeyCode::Right => {
                    self.cursor = (self.cursor + 1) % cursor_max;
                }
                KeyCode::Left => {
                    self.cursor = (self.cursor + cursor_max - 1) % cursor_max;
                }
                KeyCode::Char(c) => match c {
                    '0'..='9' => {
                        self.mask[self.cursor] = Some(c as u8 - b'0');
                    }
                    'a'..='f' => {
                        self.mask[self.cursor] = Some(c as u8 - b'a' + 10);
                    }
                    'A'..='F' => {
                        self.mask[self.cursor] = Some(c as u8 - b'A' + 10);
                    }
                    _ => {}
                },
                KeyCode::Backspace => {
                    self.mask[self.cursor] = None;
                }
                KeyCode::Enter => {}
                _ => {}
            }
        }

        Ok(result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer) -> Rect
    where
        Self: Sized,
    {
        Line::from("Create Wallet").bold().render(area, buf);

        "You can edit mask if you wish to vanity generate special address"
            .render(area.offset(Offset { x: 0, y: 3 }), buf);

        "0x".render(area.offset(Offset { x: 0, y: 5 }), buf);

        for (i, b) in self.mask.iter().enumerate() {
            let content = if let Some(n) = b {
                match n {
                    0..=9 => (b'0' + n) as char,
                    10..=15 => (b'a' + (n - 10)) as char,
                    _ => unreachable!("Only 0..=15 allowed"),
                }
            } else {
                '.'
            };
            let span = Span::from(content.to_string());

            let style = if self.cursor == i {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default()
            };

            span.style(style).render(
                area.offset(Offset {
                    x: 2 + i as i32,
                    y: 5,
                }),
                buf,
            );
        }

        "Press enter to generate address".render(area.offset(Offset { x: 0, y: 8 }), buf);

        area
    }
}
