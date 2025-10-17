use std::fmt::Display;

use crate::extensions::{MouseEventExt, ThemedWidget};
use crate::scroll_bar::CustomScrollBar;
use crate::thematize::{DefaultTheme, Thematize};

use super::cursor::Cursor;
use gm_utils::text_wrap::text_wrap;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::widgets::{List, ListItem, Widget};

struct Areas {
    list_area: Rect,
    scroll_area: Rect,
}

pub enum SelectEvent<'a, T> {
    Select(&'a T),
    Hover { on_list_area: bool },
}

struct DisplayItems<'a> {
    rendered_items: Vec<ListItem<'a>>,
    item_heights: Vec<usize>,
    start_index: usize,
    list_height: usize,
    current_page: usize,
    total_pages: usize,
}

#[derive(Debug)]
pub struct Select<T: Display + PartialEq> {
    loading_text: Option<&'static str>,
    empty_text: Option<&'static str>,
    focus: bool,
    list: Option<Vec<T>>,
    cursor: Cursor,
    hover_cursor: Option<usize>,
}

impl<T: Display + PartialEq> Default for Select<T> {
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

impl<T: Display + PartialEq> Select<T> {
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

    pub fn with_list(mut self, list: Vec<T>) -> Self {
        self.list = Some(list);
        self
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    pub fn list_push(&mut self, item: T) {
        if let Some(list) = self.list.as_mut() {
            list.push(item);
        } else {
            self.list = Some(vec![item]);
        }
    }

    pub fn list_len(&self) -> usize {
        self.list.as_ref().map(|l| l.len()).unwrap_or(0)
    }

    pub fn list_is_none(&self) -> bool {
        self.list.is_none()
    }

    pub fn get_focussed_item(&self) -> crate::Result<&T> {
        Self::get_focussed_item_internal(&self.list, self.cursor.current)
    }

    /// get focussed item without borrowing entire self
    fn get_focussed_item_internal(list: &Option<Vec<T>>, cursor: usize) -> crate::Result<&T> {
        list.as_ref()
            .ok_or(crate::Error::SelectListNotSet)
            .and_then(|list| {
                list.get(cursor).ok_or(crate::Error::SelectItemNotFound {
                    idx: cursor,
                    list_len: list.len(),
                })
            })
    }

    pub fn set_focussed_item(&mut self, item: T) {
        if let Some((index, _)) = self
            .list
            .as_ref()
            .and_then(|list| list.iter().enumerate().find(|(_, i)| **i == item))
        {
            self.cursor.current = index;
            if let Some(hover_cursor) = self.hover_cursor.as_mut() {
                *hover_cursor = index;
            }
        }
    }

    pub fn set_focus_to_last_item(&mut self) {
        if let Some(list) = self.list.as_ref() {
            if !list.is_empty() {
                self.cursor.current = list.len() - 1;
                if let Some(hover_cursor) = self.hover_cursor.as_mut() {
                    *hover_cursor = list.len() - 1;
                }
            }
        }
    }

    pub fn remove_item_at_cursor(&mut self) -> Option<T> {
        if let Some(list) = self.list.as_mut() {
            if !list.is_empty() {
                let item = list.remove(self.cursor.current);
                if self.cursor.current >= list.len() && !list.is_empty() {
                    self.cursor.current = list.len() - 1;
                }
                if let Some(hover_cursor) = self.hover_cursor.as_mut() {
                    *hover_cursor = self.cursor.current;
                }
                return Some(item);
            }
        }
        None
    }

    pub fn cursor(&self) -> usize {
        self.cursor.current
    }

    pub fn hover_cursor(&self) -> Option<usize> {
        self.hover_cursor
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor.current = cursor.min(self.list_len().saturating_sub(1));
        if let Some(hover_cursor) = self.hover_cursor.as_mut() {
            *hover_cursor = self.cursor.current;
        }
    }

    pub fn reset_cursor(&mut self) {
        self.cursor.reset();
        if let Some(hover_cursor) = self.hover_cursor.as_mut() {
            *hover_cursor = 0;
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
    ) -> crate::Result<Option<SelectEvent<'a, T>>> {
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
                            if !list.is_empty() {
                                let display = Self::display_item(
                                    list,
                                    self.cursor.current,
                                    self.hover_cursor(),
                                    self.focus,
                                    area,
                                    None::<&DefaultTheme>,
                                );

                                let mut new_cursor = display.start_index;
                                let clicked_height =
                                    mouse_event.row.saturating_sub(area.y) as usize;

                                let mut height = 0;
                                for item_height in display.item_heights {
                                    if height <= clicked_height
                                        && clicked_height < height + item_height
                                    {
                                        if mouse_event.is_left_click() {
                                            result = Some(SelectEvent::Select(
                                                Self::get_focussed_item_internal(
                                                    &self.list,
                                                    self.cursor.current,
                                                )?,
                                            ));

                                            if self.hover_cursor.is_some() {
                                                self.hover_cursor = Some(self.cursor.current);
                                            }
                                        } else if mouse_event.is(MouseEventKind::Moved) {
                                            self.cursor.current = new_cursor;

                                            result =
                                                Some(SelectEvent::Hover { on_list_area: true });
                                        }
                                    }

                                    height += item_height;
                                    new_cursor += 1;
                                }
                            }
                        }
                    } else {
                        result = Some(SelectEvent::Hover {
                            on_list_area: false,
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(result)
    }

    fn display_item<'a>(
        list: &'a [T],
        cursor: usize,
        hover_cursor: Option<usize>,
        focus: bool,
        area: Rect,
        theme: Option<&impl Thematize>,
    ) -> DisplayItems<'a> {
        let capacity = area.height as usize;
        let horizontal_layout = Layout::horizontal([Constraint::Min(3), Constraint::Length(1)]);
        let [list_area, _] = horizontal_layout.areas(area);

        let mut list_height = 0;
        let mut temp_rows = 0;
        let mut temp_rows_2 = 0;
        let mut start_index = 0;
        let mut prev_start_index = 0;
        let mut end_index = list.len() - 1;
        let mut found = false;
        let mut item_heights = vec![];

        let mut current_page: usize = 0;
        let mut total_pages: usize = 1;
        let render_item = |(i, member): (usize, &T)| {
            let text = format!("{member}");
            let wrapped_text = text_wrap(
                &text,
                if capacity < list_height {
                    list_area.width
                } else {
                    area.width
                },
            );
            list_height += wrapped_text.len();
            item_heights.push(wrapped_text.len());
            if !found {
                temp_rows += wrapped_text.len();
            }
            temp_rows_2 += wrapped_text.len();

            if temp_rows > capacity {
                temp_rows = wrapped_text.len();
                let prev_index: usize = i.saturating_sub(1);
                if cursor <= prev_index && cursor >= start_index {
                    end_index = prev_index;
                    prev_start_index = start_index;

                    found = true;
                } else {
                    current_page += 1;
                    start_index = i;
                }
            }
            if temp_rows_2 > capacity {
                temp_rows_2 = wrapped_text.len();
                total_pages += 1;
            }
            let content = Text::from(
                wrapped_text
                    .iter()
                    .map(|s| Line::from(s.clone().into_owned()))
                    .collect::<Vec<_>>(),
            );

            let style = if let Some(theme) = theme {
                if cursor == i && focus {
                    theme.select_focused()
                } else if hover_cursor.is_some_and(|hover_cursor| hover_cursor == i) {
                    theme.select_active()
                } else {
                    theme.select_inactive()
                }
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        };

        let render_items = list
            .iter()
            .enumerate()
            .map(render_item)
            .collect::<Vec<ListItem>>();

        DisplayItems {
            rendered_items: render_items[start_index..=end_index].to_vec(),
            item_heights: item_heights[start_index..=end_index].to_vec(),
            start_index,
            list_height,
            current_page,
            total_pages,
        }
    }

    fn get_areas(area: Rect) -> Areas {
        let horizontal_layout = Layout::horizontal([Constraint::Min(3), Constraint::Length(1)]);
        let [list_area, scroll_area] = horizontal_layout.areas(area);
        Areas {
            list_area,
            scroll_area,
        }
    }
}

impl<T: Display + PartialEq> ThemedWidget for Select<T> {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if let Some(list) = self.list.as_ref() {
            if !list.is_empty() {
                let display = Self::display_item(
                    list,
                    self.cursor(),
                    self.hover_cursor(),
                    self.focus,
                    area,
                    Some(theme),
                );

                let Areas {
                    list_area,
                    scroll_area,
                } = Self::get_areas(area);

                if (area.height as usize) < display.list_height {
                    List::new(display.rendered_items).render(list_area, buf);
                    CustomScrollBar {
                        cursor: display.current_page,
                        total_items: display.total_pages,
                        paginate: false,
                    }
                    .render(scroll_area, buf, theme);
                } else {
                    List::new(display.rendered_items).render(area, buf);
                }
            } else {
                self.empty_text.unwrap_or("no items").render(area, buf);
            }
        } else {
            self.loading_text.unwrap_or("Loading...").render(area, buf);
        }
    }
}
