use std::{fmt::Display, sync::Arc};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::Widget,
};

use crate::{extensions::RectExt, select_owned::SelectOwned, thematize::Thematize};

#[derive(Debug)]
pub struct FilterSelectOwned<T: Display + PartialEq> {
    pub full_list: Option<Vec<Arc<T>>>,
    pub search_string: String,
    pub select: SelectOwned<Arc<T>>,
}

impl<T: Display + PartialEq> FilterSelectOwned<T> {
    pub fn new(items: Option<Vec<T>>) -> Self {
        let mut filter_select = Self {
            full_list: None,
            search_string: String::new(),
            select: SelectOwned::default(),
        };
        filter_select.set_items(items.map(|items| items.into_iter().map(Arc::new).collect()));
        filter_select
    }

    pub fn set_items(&mut self, items: Option<Vec<Arc<T>>>) {
        self.full_list = items;

        self.update_select_list();
    }

    pub fn update_select_list(&mut self) {
        self.select.cursor.current = 0;
        self.select.list = self.full_list.as_ref().map(|full_list| {
            full_list
                .iter()
                .filter(|entry| format!("{entry}").contains(self.search_string.as_str()))
                .cloned()
                .collect()
        });
    }

    pub fn handle_event<E, F>(
        &mut self,
        input_event: Option<&Event>,
        area: Rect,
        on_enter: F,
    ) -> Result<(), E>
    where
        F: FnMut(&Arc<T>) -> Result<(), E>,
    {
        if self.full_list.is_some() {
            if let Some(list_area) = area.height_consumed(2) {
                self.select.handle_event(input_event, list_area, on_enter)?;
            }

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
        }

        Ok(())
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        let horizontal_layout = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]);
        let [search_area, list_area] = horizontal_layout.areas(area);
        Line::from(if self.search_string.is_empty() {
            "Type to filter".to_string()
        } else {
            format!("Filter: {}", self.search_string)
        })
        .render(search_area, buf);

        self.select.render(list_area, buf, theme);
    }
}
