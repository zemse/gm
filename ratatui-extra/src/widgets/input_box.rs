use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    layout::{Offset, Position, Rect},
    style::Modifier,
    text::Span,
    widgets::{Block, Widget},
};

use crate::{
    extensions::{MouseEventExt, RectExt, WidgetHeight},
    thematize::Thematize,
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
    pub fn handle_event(
        input_event: Option<&Event>,
        area: Rect,
        text_input: &mut String,
        text_cursor: &mut usize,
    ) -> bool {
        if let Some(input_event) = input_event {
            match input_event {
                Event::Key(key_event) => match key_event.code {
                    KeyCode::Left => {
                        if key_event.modifiers == KeyModifiers::ALT {
                            option_left(text_input, text_cursor);
                        } else if *text_cursor > 0 {
                            *text_cursor -= 1
                        }
                        return true;
                    }
                    KeyCode::Right => {
                        if key_event.modifiers == KeyModifiers::ALT {
                            option_right(text_input, text_cursor);
                        } else if *text_cursor < text_input.len() {
                            *text_cursor += 1
                        }
                        return true;
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
                        else if key_event.modifiers == KeyModifiers::NONE
                            || key_event.modifiers == KeyModifiers::SHIFT
                        {
                            text_input.insert(*text_cursor, char);
                            *text_cursor += 1;
                        }
                        return true;
                    }
                    KeyCode::Backspace => {
                        if key_event.modifiers == KeyModifiers::ALT {
                            option_delete(text_input, text_cursor);
                        } else if *text_cursor > 0 {
                            *text_cursor -= 1;
                            text_input.remove(*text_cursor);
                        }
                        return true;
                    }
                    _ => {}
                },
                Event::Mouse(mouse_event) => {
                    if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                        let lines = split_string(text_input, (area.width - 4) as usize);
                        let area_text = Rect {
                            x: area.x,
                            y: area.y,
                            width: area.width,
                            height: (2 + lines.len()) as u16,
                        }
                        .block_inner()
                        .margin_h(1);
                        if area_text.contains(mouse_event.position()) {
                            let relative_x =
                                mouse_event.column.saturating_sub(area_text.x) as usize;
                            let relative_y = mouse_event.row.saturating_sub(area_text.y) as usize;
                            let new_cursor = relative_x + relative_y * (area.width - 4) as usize;

                            *text_cursor = new_cursor.min(text_input.len());
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }

    pub fn render(self, area: Rect, buf: &mut Buffer, text_cursor: &usize, theme: &impl Thematize)
    where
        Self: Sized,
    {
        let lines = split_string(self.text, (area.width - 4) as usize);
        let area_used = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: (2 + lines.len()) as u16,
        };

        let inner_area = if theme.boxed() {
            let block = Block::bordered()
                .border_type(theme.border_type())
                .title(self.label);
            let inner_area = block.inner(area_used);
            block.render(area_used, buf);
            inner_area.margin_h(1)
        } else {
            Span::raw(self.label)
                .style(theme.style_dim())
                .render(area_used, buf);
            Span::raw(">")
                .style(theme.style())
                .render(area_used.margin_top(1), buf);
            area_used.block_inner().margin_h(1)
        };

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

        if self.text.is_empty() && self.empty_text.is_some() {
            self.empty_text.unwrap().render(inner_area, buf);
        }
        if self.focus {
            let cx = inner_area.x + (*text_cursor as u16) % (area.width - 4);
            let cy = inner_area.y + (*text_cursor as u16) / (area.width - 4);

            let Some(cell) = buf.cell_mut(Position::new(cx, cy)) else {
                return;
            };

            if cell.symbol().is_empty() {
                cell.set_symbol(" ");
            }

            let cur_style = cell.style();
            cell.set_style(cur_style.add_modifier(Modifier::REVERSED));
        }
    }
}

impl WidgetHeight for InputBox<'_> {
    fn height_used(&self, area: ratatui::prelude::Rect) -> u16 {
        let lines = split_string(self.text, (area.width - 2) as usize);
        (2 + lines.len()) as u16
    }
}
