use std::fmt::Display;

use crate::extensions::MouseEventExt;
use crate::select::Select;
use crate::thematize::Thematize;

use super::cursor::Cursor;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

#[derive(Debug)]
pub struct SelectOwned<T: Display + PartialEq> {
    pub focus: bool,
    pub list: Option<Vec<T>>,
    pub cursor: Cursor,
}

impl<T: Display + PartialEq> Default for SelectOwned<T> {
    fn default() -> Self {
        Self {
            focus: false,
            list: None,
            cursor: Cursor::default(),
        }
    }
}

impl<T: Display + PartialEq> SelectOwned<T> {
    pub fn new(list: Option<Vec<T>>) -> Self {
        Self {
            focus: false,
            list,
            cursor: Cursor::default(),
        }
    }

    /// Returns true if the list is Some and empty
    pub fn is_some_empty(&self) -> bool {
        self.list.as_ref().map(|l| l.is_empty()).unwrap_or_default()
    }

    pub fn get_focussed_item(&self) -> Option<&T> {
        self.list
            .as_ref()
            .and_then(|list| list.get(self.cursor.current))
    }

    pub fn set_focussed_item(&mut self, item: T) {
        if let Some((index, _)) = self
            .list
            .as_ref()
            .and_then(|list| list.iter().enumerate().find(|(_, i)| **i == item))
        {
            self.cursor.current = index;
        }
    }

    pub fn update_list(&mut self, new_list: Option<Vec<T>>) {
        self.list = new_list;

        if let Some(new_list) = self.list.as_ref() {
            if self.cursor.current >= new_list.len() {
                self.cursor.current = new_list.len().saturating_sub(1);
            }
        } else {
            self.cursor.current = 0;
        }
    }

    pub fn handle_event<E, F>(
        &mut self,
        input_event: Option<&Event>,
        area: Rect,
        mut on_select: F,
    ) -> Result<(), E>
    where
        F: FnMut(&T) -> Result<(), E>,
    {
        if let Some(input_event) = input_event {
            match input_event {
                Event::Key(key_event) => {
                    if let Some(list) = self.list.as_ref() {
                        self.cursor.handle(Some(key_event), list.len());

                        if key_event.kind == KeyEventKind::Press {
                            #[allow(clippy::single_match)]
                            match key_event.code {
                                KeyCode::Enter => {
                                    if !list.is_empty() {
                                        on_select(&list[self.cursor.current])?;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Event::Mouse(mouse_event) => {
                    if MouseEventKind::Down(MouseButton::Left) == mouse_event.kind
                        && area.contains(mouse_event.position())
                    {
                        if let Some(list) = self.list.as_ref() {
                            let (_, item_heights, start_index, _, _, _) = Select {
                                list,
                                cursor: &self.cursor,
                                focus: true,
                                focus_style: Style::default(),
                            }
                            .display_item(area);

                            let mut new_cursor = start_index;
                            let clicked_height = mouse_event.row.saturating_sub(area.y) as usize;

                            let mut height = 0;
                            for item_height in item_heights {
                                if height <= clicked_height && clicked_height < height + item_height
                                {
                                    if self.cursor.current == new_cursor {
                                        on_select(&list[self.cursor.current])?;
                                    } else {
                                        self.cursor.current = new_cursor;
                                    }
                                }

                                height += item_height;
                                new_cursor += 1;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if let Some(list) = self.list.as_ref() {
            Select {
                list,
                cursor: &self.cursor,
                focus: true,
                focus_style: theme.select_focused(),
            }
            .render(area, buf);
        } else {
            "no items".render(area, buf);
        }
    }
}
