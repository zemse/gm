use std::fmt::Display;

use crate::extensions::{MouseEventExt, ThemedWidget};
use crate::select::Select;
use crate::thematize::{DefaultTheme, Thematize};

use super::cursor::Cursor;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

pub enum SelectEvent<'a, T> {
    Select(&'a T),
    Hover { on_list_area: bool },
}

#[derive(Debug)]
pub struct SelectOwned<T: Display + PartialEq> {
    /// If true, the item at cursor is applied with select_focused style from Thematize
    pub focus: bool,
    pub list: Option<Vec<T>>,
    pub cursor: Cursor,
    pub external_cursor: Option<usize>,
}

impl<T: Display + PartialEq> Default for SelectOwned<T> {
    fn default() -> Self {
        Self {
            focus: false,
            list: None,
            cursor: Cursor::new(0),
            external_cursor: None,
        }
    }
}

impl<T: Display + PartialEq> SelectOwned<T> {
    pub fn new(list: Option<Vec<T>>, external_cursor: bool) -> Self {
        Self {
            focus: false,
            list,
            cursor: Cursor::new(0),
            external_cursor: external_cursor.then_some(0),
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.list.as_ref().map(|l| l.len()).unwrap_or(0)
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
            if let Some(external_cursor) = self.external_cursor.as_mut() {
                *external_cursor = index;
            }
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

        if let Some(external_cursor) = self.external_cursor.as_mut() {
            *external_cursor = self.cursor.current;
        }
    }

    pub fn handle_event<'a>(
        &'a mut self,
        input_event: Option<&Event>,
        area: Rect,
    ) -> Option<SelectEvent<'a, T>> {
        let mut result = None;

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
                                        result =
                                            Some(SelectEvent::Select(&list[self.cursor.current]));

                                        if self.external_cursor.is_some() {
                                            self.external_cursor = Some(self.cursor.current);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Event::Mouse(mouse_event) => {
                    if area.contains(mouse_event.position()) {
                        if let Some(list) = self.list.as_ref() {
                            let (_, item_heights, start_index, _, _, _) = Select {
                                list,
                                cursor: &self.cursor,
                                focus: true,
                            }
                            .display_item(area, self.external_cursor, None::<&DefaultTheme>);

                            let mut new_cursor = start_index;
                            let clicked_height = mouse_event.row.saturating_sub(area.y) as usize;

                            let mut height = 0;
                            for item_height in item_heights {
                                if height <= clicked_height && clicked_height < height + item_height
                                {
                                    if mouse_event.is_left_click() {
                                        // on_select(&list[self.cursor.current])?;
                                        result =
                                            Some(SelectEvent::Select(&list[self.cursor.current]));

                                        if self.external_cursor.is_some() {
                                            self.external_cursor = Some(self.cursor.current);
                                        }
                                    } else if mouse_event.is(MouseEventKind::Moved) {
                                        self.cursor.current = new_cursor;
                                        // on_hover(true)?;
                                        result = Some(SelectEvent::Hover { on_list_area: true });
                                    }
                                }

                                height += item_height;
                                new_cursor += 1;
                            }
                        }
                    } else {
                        // on_hover(false)?;
                        result = Some(SelectEvent::Hover {
                            on_list_area: false,
                        });
                    }
                }
                _ => {}
            }
        }

        result
    }
}

impl<T: Display + PartialEq> ThemedWidget for SelectOwned<T> {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if let Some(list) = self.list.as_ref() {
            Select {
                list,
                cursor: &self.cursor,
                focus: self.focus,
            }
            .render(area, buf, self.external_cursor, theme);
        } else {
            "no items".render(area, buf);
        }
    }
}
