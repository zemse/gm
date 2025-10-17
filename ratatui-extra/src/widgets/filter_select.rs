use std::{fmt::Display, sync::Arc};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::WidgetRef,
};

use crate::{
    extensions::{RectExt, ThemedWidget},
    select::{Select, SelectEvent},
    thematize::Thematize,
};

#[derive(Debug)]
pub struct FilterSelect<T: Display + PartialEq> {
    pub full_list: Option<Vec<Arc<T>>>,
    pub search_string: String,
    pub select: Select<Arc<T>>,
}

impl<T: Display + PartialEq> Default for FilterSelect<T> {
    fn default() -> Self {
        Self {
            full_list: None,
            search_string: String::default(),
            select: Select::default(),
        }
    }
}

impl<T: Display + PartialEq> FilterSelect<T> {
    pub fn with_empty_text(mut self, empty_text: &'static str) -> Self {
        self.select = self.select.with_empty_text(empty_text);
        self
    }

    pub fn with_items(mut self, items: Vec<T>) -> Self {
        self.set_items(Some(items));
        self
    }

    pub fn with_focus(mut self, focus: bool) -> Self {
        self.select.set_focus(focus);
        self
    }

    pub fn set_items(&mut self, items: Option<Vec<T>>) {
        self.full_list = items.map(|items| items.into_iter().map(Arc::new).collect());

        self.update_select_list();
    }

    pub fn update_select_list(&mut self) {
        self.select
            .update_list(self.full_list.as_ref().map(|full_list| {
                full_list
                    .iter()
                    .filter(|entry| format!("{entry}").contains(self.search_string.as_str()))
                    .cloned()
                    .collect()
            }));
    }

    pub fn get_focussed_item(&self) -> crate::Result<&Arc<T>> {
        self.select.get_focussed_item()
    }

    pub fn reset(&mut self) {
        self.select.reset_cursor();
        self.search_string.clear();
    }

    pub fn list_len(&self) -> usize {
        self.select.list_len()
    }

    pub fn handle_event<'a>(
        &'a mut self,
        input_event: Option<&Event>,
        area: Rect,
    ) -> crate::Result<Option<SelectEvent<'a, Arc<T>>>> {
        let mut result: Option<SelectEvent<'_, Arc<T>>> = None;

        if self.full_list.is_some() {
            let search_string_prev = self.search_string.clone();
            if let Some(Event::Key(key_event)) = input_event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char(char) => {
                            self.search_string.push(char);
                        }
                        KeyCode::Backspace => {
                            self.search_string.pop();
                        }
                        _ => {}
                    }
                }
            }

            if search_string_prev != self.search_string {
                self.update_select_list();
            }

            result = self.select.handle_event(input_event, area.margin_top(2))?;
        }

        Ok(result)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if self.select.list_len() > 0 {
            let horizontal_layout = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]);
            let [search_area, list_area] = horizontal_layout.areas(area);
            Line::from(if self.search_string.is_empty() {
                "Type to filter".to_string()
            } else {
                format!("Filter: {}", self.search_string)
            })
            .render_ref(search_area, buf);

            self.select.render(list_area, buf, theme);
        } else {
            self.select.render(area, buf, theme);
        }
    }
}
