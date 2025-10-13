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
    loading_text: Option<&'static str>,
    empty_text: Option<&'static str>,
    focus: bool,
    list: Option<Vec<T>>,
    cursor: Cursor,
    hover_cursor: Option<usize>,
}

impl<T: Display + PartialEq> Default for SelectOwned<T> {
    fn default() -> Self {
        Self {
            loading_text: None,
            empty_text: None,
            focus: false,
            list: None,
            cursor: Cursor::new(0),
            hover_cursor: None,
        }
    }
}

impl<T: Display + PartialEq> SelectOwned<T> {
    // TODO remove this function and use builder pattern
    pub fn new(list: Option<Vec<T>>, hover_cursor: bool) -> Self {
        Self {
            loading_text: None,
            empty_text: None,
            focus: false,
            list,
            cursor: Cursor::new(0),
            hover_cursor: hover_cursor.then_some(0),
        }
    }

    pub fn with_loading_text(mut self, loading_text: &'static str) -> Self {
        self.loading_text = Some(loading_text);
        self
    }

    pub fn with_empty_text(mut self, empty_text: &'static str) -> Self {
        self.empty_text = Some(empty_text);
        self
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    pub fn list_len(&self) -> usize {
        self.list.as_ref().map(|l| l.len()).unwrap_or(0)
    }

    pub fn list_is_none(&self) -> bool {
        self.list.is_none()
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
            if let Some(external_cursor) = self.hover_cursor.as_mut() {
                *external_cursor = index;
            }
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor.current
    }

    pub fn hover_cursor(&self) -> Option<usize> {
        self.hover_cursor
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor.current = cursor.min(self.list_len().saturating_sub(1));
        if let Some(external_cursor) = self.hover_cursor.as_mut() {
            *external_cursor = self.cursor.current;
        }
    }

    pub fn reset_cursor(&mut self) {
        self.cursor.reset();
        if let Some(external_cursor) = self.hover_cursor.as_mut() {
            *external_cursor = 0;
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

        if let Some(hover_cursor) = self.hover_cursor.as_mut() {
            *hover_cursor = self.cursor.current;
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

                                        if self.hover_cursor.is_some() {
                                            self.hover_cursor = Some(self.cursor.current);
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
                            .display_item(area, self.hover_cursor, None::<&DefaultTheme>);

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

                                        if self.hover_cursor.is_some() {
                                            self.hover_cursor = Some(self.cursor.current);
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
            if !list.is_empty() {
                Select {
                    list,
                    cursor: &self.cursor,
                    focus: self.focus,
                }
                .render(area, buf, self.hover_cursor, theme);
            } else {
                self.empty_text.unwrap_or("no items").render(area, buf);
            }
        } else {
            self.loading_text.unwrap_or("Loading...").render(area, buf);
        }
    }
}
