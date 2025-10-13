use std::fmt::Display;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::Rect,
    text::Span,
    widgets::{Block, Widget},
};

use super::{filter_select::FilterSelect, popup::Popup};
use crate::{
    act::Act,
    cursor::Cursor,
    extensions::{KeyEventExt, RectExt},
    thematize::Thematize,
};

// TODO use FilterSelectOwned instead of FilterSelect
#[derive(Clone, Debug)]
pub struct FilterSelectPopup<Item: Display + Clone> {
    title: &'static str,
    empty_text: Option<&'static str>,
    open: bool,
    items: Option<Vec<Item>>,
    cursor: Cursor,
    search_string: String,
}

impl<Item: Display + Clone> FilterSelectPopup<Item> {
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

    pub fn display_selection(&self) -> String {
        self.current_selection()
            .map(|s| s.to_string())
            .unwrap_or_default()
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

    pub fn handle_event<'a, A>(
        &'a mut self,
        key_event: Option<&KeyEvent>,
        actions: &mut A,
    ) -> Option<&'a Item>
    where
        A: Act,
    {
        let mut result = None;

        if self.open {
            if key_event.is_pressed(KeyCode::Esc) {
                self.close();
            }

            if self.items.is_some() {
                let cursor_max = self
                    .items
                    .as_ref()
                    .map(|items| items.len())
                    .unwrap_or_else(|| unreachable!());
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
                                self.close();
                                let items = self.items.as_ref().unwrap();
                                result = Some(&items[self.cursor.current]);
                            }
                            _ => {}
                        }
                    }
                }
            }

            actions.ignore_esc();
        }

        result
    }
    pub fn render(&self, popup_area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            Popup.render(popup_area, buf, &theme);

            if theme.boxed() {
                let block = Block::bordered()
                    .border_type(theme.border_type())
                    .style(theme.style());
                block.render(popup_area, buf);
            }

            let mut inner_area = Popup::inner_area(popup_area);

            Span::raw(self.title).render(inner_area, buf);
            inner_area.consume_height(2);

            if let Some(items) = &self.items {
                if items.is_empty() {
                    if let Some(empty_text) = self.empty_text {
                        empty_text.render(inner_area, buf);
                    } else {
                        "The list is empty.".render(inner_area, buf);
                    }
                } else {
                    FilterSelect {
                        full_list: items,
                        cursor: &self.cursor,
                        search_string: &self.search_string,
                        focus: true,
                    }
                    .render(inner_area, buf, &theme);
                }
            } else {
                "Loading...".render(inner_area, buf);
            }
        }
    }
}
