use std::cmp::min;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Offset, Rect},
    text::Span,
    widgets::{Block, Widget},
};

use crate::tui::{
    traits::{HandleResult, WidgetHeight},
    Event,
};

pub struct InputBox<'a> {
    pub focus: bool,
    pub label: &'static str,
    pub text: &'a String,
    pub empty_text: Option<&'static str>,
}

impl InputBox<'_> {
    pub fn handle_events(text_input: &mut String, event: &Event) -> crate::Result<HandleResult> {
        let result = HandleResult::default();

        if let Event::Input(key_event) = event {
            match key_event.code {
                KeyCode::Char(char) => {
                    // Handle space key on empty state
                    if text_input.is_empty() && char == ' ' {
                        // Ignore leading spaces
                    } else
                    // Handle command + delete on macOS
                    if char == 'u' && key_event.modifiers == KeyModifiers::CONTROL {
                        text_input.clear();
                    } else
                    // Handle option + delete on macOS
                    if char == 'w' && key_event.modifiers == KeyModifiers::CONTROL {
                        loop {
                            let char = text_input.pop();
                            if char.is_none() || char == Some(' ') {
                                break;
                            }
                        }
                    } else {
                        text_input.push(char);
                    }
                }
                KeyCode::Backspace => {
                    text_input.pop();
                }
                _ => {}
            }
        }

        Ok(result)
    }
}

impl Widget for InputBox<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let lines = split_string(self.text, (area.width - 2) as usize);
        let area_used = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: (2 + lines.len()) as u16,
        };

        let block = Block::bordered().title(self.label);
        let inner_area = block.inner(area_used);
        block.render(area_used, buf);

        if self.focus {
            if self.text.is_empty() && self.empty_text.is_some() {
                self.empty_text.unwrap().render(inner_area, buf);
            } else {
                Span::from("|").render(
                    Rect {
                        x: inner_area.x + lines.last().unwrap().len() as u16,
                        y: inner_area.y + lines.len() as u16 - 1,
                        width: 1,
                        height: 1,
                    },
                    buf,
                );
            }
        }

        for (idx, line) in lines.into_iter().enumerate() {
            line.render(
                inner_area.offset(Offset {
                    x: 0,
                    y: idx as i32,
                }),
                buf,
            );
        }
    }
}

impl WidgetHeight for InputBox<'_> {
    fn height_used(&self, area: ratatui::prelude::Rect) -> u16 {
        let lines = split_string(self.text, (area.width - 2) as usize);
        (2 + lines.len()) as u16
    }
}

fn split_string(s: &str, max_width: usize) -> Vec<&str> {
    let mut lines = vec![];

    let mut ptr = 0;
    let s_len = s.len();
    while ptr < s_len {
        let next = min(ptr + max_width, s_len);
        let s = s.get(ptr..next).expect("couldnt slice"); // can't go wrong
        lines.push(s);
        ptr = next;
    }

    if lines.is_empty() {
        lines.push("");
    }

    lines
}

#[cfg(test)]
mod test {
    use crate::tui::app::widgets::input_box::split_string;

    #[test]
    fn test_split_string() {
        assert_eq!(
            split_string("hello what is up", 6),
            vec!["hello ", "what i", "s up"]
        );
    }
}
