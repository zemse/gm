use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    layout::{Offset, Position, Rect},
    style::Stylize,
    text::Span,
    widgets::{Block, Widget},
};

use crate::{
    act::Act,
    event::WidgetEvent,
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

#[derive(Debug)]
pub struct InputBoxOwned {
    pub label: &'static str,
    text_input: String,
    text_cursor: usize,
    cursor_blink_visible: bool,
    last_move: Instant,
    empty_text: Option<&'static str>,
    pub currency: Option<String>,
    is_immutable: bool,
}

impl InputBoxOwned {
    // TODO use builder pattern instead of many args
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            text_input: String::new(),
            text_cursor: 0,
            cursor_blink_visible: true,
            last_move: Instant::now(),
            empty_text: None,
            currency: None,
            is_immutable: false,
        }
    }

    pub fn with_empty_text(mut self, empty_text: &'static str) -> Self {
        self.empty_text = Some(empty_text);
        self
    }

    pub fn with_currency(mut self, currency: String) -> Self {
        self.currency = Some(currency);
        self
    }

    pub fn make_immutable(mut self, immutable: bool) -> Self {
        self.is_immutable = immutable;
        self
    }

    pub fn get_text(&self) -> &str {
        &self.text_input
    }

    pub fn set_text(&mut self, text: String) {
        self.text_input = text;
        self.text_cursor = self.text_input.len();
    }

    pub fn handle_event<A: Act>(
        &mut self,
        widget_event: Option<&WidgetEvent>,
        area: Rect,
        actions: &mut A,
    ) {
        match widget_event {
            Some(WidgetEvent::Tick) => {
                self.cursor_blink_visible = !self.cursor_blink_visible;
            }
            Some(WidgetEvent::InputEvent(input_event)) => {
                // Only handle input events if this box is not immutable
                if !self.is_immutable {
                    match input_event {
                        Event::Key(key_event) => match key_event.code {
                            KeyCode::Left => {
                                if key_event.modifiers == KeyModifiers::ALT {
                                    option_left(&self.text_input, &mut self.text_cursor);
                                    actions.ignore_left();
                                    self.last_move = Instant::now();
                                } else if self.text_cursor > 0 {
                                    self.text_cursor -= 1;
                                    actions.ignore_left();
                                    self.last_move = Instant::now();
                                }
                            }
                            KeyCode::Right => {
                                if key_event.modifiers == KeyModifiers::ALT {
                                    option_right(&self.text_input, &mut self.text_cursor);
                                    actions.ignore_right();
                                    self.last_move = Instant::now();
                                } else if self.text_cursor < self.text_input.len() {
                                    self.text_cursor += 1;
                                    actions.ignore_right();
                                    self.last_move = Instant::now();
                                }
                            }
                            KeyCode::Char(char) => {
                                // Handle space key on empty state
                                if self.text_input.is_empty() && char == ' ' {
                                    // Ignore leading spaces
                                    self.last_move = Instant::now();
                                }
                                // Handle command + delete on macOS
                                else if char == 'u'
                                    && key_event.modifiers == KeyModifiers::CONTROL
                                {
                                    let (_, right) = self.text_input.split_at(self.text_cursor);
                                    self.text_input = right.to_string();
                                    self.text_cursor = 0;
                                    self.last_move = Instant::now();
                                }
                                // Handle command + left on macOS
                                else if char == 'a'
                                    && key_event.modifiers == KeyModifiers::CONTROL
                                {
                                    self.text_cursor = 0;
                                    self.last_move = Instant::now();
                                }
                                // Handle command + right on macOS
                                else if char == 'e'
                                    && key_event.modifiers == KeyModifiers::CONTROL
                                {
                                    self.text_cursor = self.text_input.len();
                                    self.last_move = Instant::now();
                                }
                                // Handle option + delete on macOS
                                else if char == 'w'
                                    && key_event.modifiers == KeyModifiers::CONTROL
                                {
                                    option_delete(&mut self.text_input, &mut self.text_cursor);
                                    self.last_move = Instant::now();
                                }
                                // option + Left
                                else if char == 'b' && key_event.modifiers == KeyModifiers::ALT {
                                    option_left(&self.text_input, &mut self.text_cursor);
                                    self.last_move = Instant::now();
                                }
                                // option + Right
                                else if char == 'f' && key_event.modifiers == KeyModifiers::ALT {
                                    option_right(&self.text_input, &mut self.text_cursor);
                                    self.last_move = Instant::now();
                                }
                                // Simple char press
                                else if key_event.modifiers == KeyModifiers::NONE
                                    || key_event.modifiers == KeyModifiers::SHIFT
                                {
                                    self.text_input.insert(self.text_cursor, char);
                                    self.text_cursor += 1;
                                    self.last_move = Instant::now();
                                }
                            }
                            KeyCode::Backspace => {
                                if key_event.modifiers == KeyModifiers::ALT {
                                    option_delete(&mut self.text_input, &mut self.text_cursor);
                                    self.last_move = Instant::now();
                                } else if self.text_cursor > 0 {
                                    self.text_cursor -= 1;
                                    self.text_input.remove(self.text_cursor);
                                    self.last_move = Instant::now();
                                }
                            }
                            _ => {}
                        },
                        Event::Mouse(mouse_event) => {
                            if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                                let lines =
                                    split_string(&self.text_input, (area.width - 4) as usize);
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
                                    let relative_y =
                                        mouse_event.row.saturating_sub(area_text.y) as usize;
                                    let new_cursor =
                                        relative_x + relative_y * (area.width - 4) as usize;

                                    self.text_cursor = new_cursor.min(self.text_input.len());
                                    self.last_move = Instant::now();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, focus: bool, theme: &impl Thematize)
    where
        Self: Sized,
    {
        let lines = split_string(&self.text_input, (area.width - 4) as usize);
        let area_used = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: (2 + lines.len()) as u16,
        };

        let inner_area = if theme.boxed() {
            Block::bordered()
                .border_type(theme.border_type())
                .style(if focus {
                    theme.style()
                } else {
                    theme.style_dim()
                })
                .title(self.label)
                .render(area_used, buf);
            area_used.block_inner().margin_h(1)
        } else {
            Span::raw(self.label)
                .style(theme.style_dim())
                .render(area_used, buf);
            Span::raw(">")
                .style(if focus {
                    theme.style().bold()
                } else {
                    theme.style_dim()
                })
                .render(area_used.margin_top(1), buf);
            area_used.block_inner().margin_h(1)
        };

        if lines.len() == 1 && !lines.last().unwrap().is_empty() && self.currency.is_some() {
            let currency = self.currency.as_ref().unwrap();
            Span::from(currency).render(
                inner_area.offset(Offset {
                    x: lines.last().unwrap().len() as i32 + 1,
                    y: 0,
                }),
                buf,
            );
        }

        let cursor_blink_visible =
            self.last_move.elapsed().lt(&Duration::from_millis(500)) || self.cursor_blink_visible;

        for (idx, line) in lines.into_iter().enumerate() {
            let style = if focus {
                if self.is_immutable {
                    theme.cursor()
                } else {
                    theme.style()
                }
            } else {
                theme.style_dim()
            };

            Span::raw(line).style(style).render(
                inner_area.offset(Offset {
                    x: 0,
                    y: idx as i32,
                }),
                buf,
            );
        }

        if self.text_input.is_empty() && self.empty_text.is_some() {
            Span::raw(self.empty_text.unwrap())
                .style(theme.style_dim())
                .render(inner_area, buf);
        }

        if focus && cursor_blink_visible && !self.is_immutable {
            let cx = inner_area.x + (self.text_cursor as u16) % (area.width - 4);
            let cy = inner_area.y + (self.text_cursor as u16) / (area.width - 4);

            let Some(cell) = buf.cell_mut(Position::new(cx, cy)) else {
                return;
            };

            if cell.symbol().is_empty() {
                cell.set_symbol(" ");
            }

            cell.set_style(theme.cursor());
        }
    }
}

// TODO this is the right way to get height used, remove the Rect returns from render
impl WidgetHeight for InputBoxOwned {
    fn height_used(&self, area: ratatui::prelude::Rect) -> u16 {
        let lines = split_string(&self.text_input, (area.width - 2) as usize);
        (2 + lines.len()) as u16
    }
}
