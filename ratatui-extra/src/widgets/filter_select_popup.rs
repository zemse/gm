use std::fmt::Display;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    widgets::{Block, Widget},
};

use super::{filter_select::FilterSelect, popup::Popup};
use crate::{act::Act, cursor::Cursor, extensions::KeyEventExt, thematize::Thematize};

#[derive(Clone, Debug)]
pub struct FilterSelectPopup<Item: Display> {
    title: &'static str,
    empty_text: Option<&'static str>,
    open: bool,
    items: Option<Vec<Item>>,
    cursor: Cursor,
    search_string: String,
}

impl<Item: Display> FilterSelectPopup<Item> {
    pub fn new(title: &'static str, empty_text: Option<&'static str>) -> Self {
        Self {
            title,
            empty_text,
            open: false,
            items: None,
            cursor: Cursor::new(0),
            search_string: String::new(),
        }
    }

    pub fn set_items(&mut self, items: Option<Vec<Item>>) {
        self.items = items;
    }

    pub fn set_cursor(&mut self, item: &Item) {
        if let Some(items) = &self.items {
            if let Some(index) = items.iter().position(|i| i.to_string() == item.to_string()) {
                self.cursor.current = index;
            }
        }
    }

    pub fn current_selection(&self) -> Option<&Item> {
        self.items
            .as_ref()
            .and_then(|items| items.get(self.cursor.current))
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    // Opens the popup with the fresh items.
    pub fn open(&mut self) {
        self.open = true;
        self.cursor.reset();
        self.search_string.clear();
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn handle_event<A, E, F>(
        &mut self,
        key_event: Option<&KeyEvent>,
        mut on_enter: F,
    ) -> Result<A, E>
    where
        A: Act,
        F: FnMut(&Item) -> Result<(), E>,
    {
        let mut act = A::default();

        if self.open {
            if self.items.is_some() {
                let items = self.items.as_ref().unwrap();
                let cursor_max = items.len();
                self.cursor.handle(key_event, cursor_max);

                if let Some(key_event) = key_event {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char(char) => {
                                self.search_string.push(char);
                            }
                            KeyCode::Backspace => {
                                self.search_string.pop();
                            }
                            KeyCode::Enter => {
                                on_enter(&items[self.cursor.current])?;
                                self.close();
                            }
                            _ => {}
                        }
                    }
                }
            }

            if key_event.is_pressed(KeyCode::Esc) {
                self.close();
            }

            act.ignore_esc();
        }

        Ok(act)
    }
    pub fn render(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        theme: &impl Thematize,
    ) where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            Popup.render(area, buf, &theme);

            let inner_area = Popup::inner_area(area);
            let block = Block::bordered()
                .border_type(theme.border_type())
                .style(theme.style())
                .title(self.title);
            let block_inner_area = block.inner(inner_area);
            block.render(inner_area, buf);

            if let Some(items) = &self.items {
                if items.is_empty() {
                    if let Some(empty_text) = self.empty_text {
                        empty_text.render(block_inner_area, buf);
                    } else {
                        "The list is empty.".render(block_inner_area, buf);
                    }
                } else {
                    FilterSelect {
                        full_list: items,
                        cursor: &self.cursor,
                        search_string: &self.search_string,
                        focus: true,
                    }
                    .render(block_inner_area, buf, &theme);
                }
            } else {
                "Loading...".render(block_inner_area, buf);
            }
        }
    }
}
