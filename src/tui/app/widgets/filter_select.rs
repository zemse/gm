use std::fmt::Display;

use ratatui::{
    layout::{Constraint, Layout},
    text::Line,
    widgets::Widget,
};

use crate::utils::cursor::Cursor;

use super::select::Select;

pub struct FilterSelect<'a, T: Display> {
    pub full_list: &'a Vec<T>,
    pub cursor: &'a Cursor,
    pub search_string: &'a String,
    pub focus: bool,
}

impl<T: Display> Widget for FilterSelect<'_, T> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
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
        Select {
            list: &self
                .full_list
                .iter()
                .filter(|item| item.to_string().contains(self.search_string))
                .collect::<Vec<&T>>(),
            cursor: self.cursor,
            focus: self.focus,
        }
        .render(list_area, buf);
    }
}
