use std::fmt::Display;

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    style::Color,
    widgets::{Block, Widget},
};

use crate::{
    tui::{traits::HandleResult, Event},
    utils::cursor::Cursor,
};

use super::{filter_select::FilterSelect, popup::Popup};

pub struct FilterSelectPopup<Item: Display> {
    title: &'static str,
    open: bool,
    items: Vec<Item>,
    cursor: Cursor,
    search_string: String,
}

impl<Item: Display> FilterSelectPopup<Item> {
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            open: false,
            items: vec![],
            cursor: Cursor::default(),
            search_string: String::new(),
        }
    }
    pub fn is_open(&self) -> bool {
        self.open
    }

    // Opens the popup with the fresh items.
    pub fn open(&mut self, items: Vec<Item>) {
        self.open = true;
        self.cursor.reset();
        self.search_string.clear();
        self.items = items;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn handle_event<F>(
        &mut self,
        event: &crate::tui::Event,
        mut on_enter: F,
    ) -> crate::Result<HandleResult>
    where
        F: FnMut(&Item),
    {
        let mut result = HandleResult::default();

        if self.open {
            let cursor_max = self.items.len();
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
                            on_enter(&self.items[self.cursor.current]);
                            self.close();
                        }
                        _ => {}
                    }
                }
            }

            if event.is_key_pressed(KeyCode::Esc) {
                self.close();
            }

            result.esc_ignores = 1;
        }

        Ok(result)
    }
}

impl<Item: Display> Widget for &FilterSelectPopup<Item> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.is_open() {
            Popup {
                bg_color: Some(Color::Blue),
            }
            .render(area, buf);

            let inner_area = Popup::inner_area(area);
            let block = Block::bordered().title(self.title);
            let block_inner_area = block.inner(inner_area);
            block.render(inner_area, buf);

            FilterSelect {
                full_list: &self.items,
                cursor: &self.cursor,
                search_string: &self.search_string,
                focus: true,
            }
            .render(block_inner_area, buf);
        }
    }
}
