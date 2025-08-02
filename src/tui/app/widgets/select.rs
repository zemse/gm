use std::fmt::{format, Display};

use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    text::Line,
    widgets::{List, ListItem, Widget},
};
use ratatui::text::{Text, ToText};
use crate::utils::cursor::Cursor;

use super::scroll_bar::CustomScrollBar;

pub struct Select<'a, T: Display> {
    pub focus: bool,
    pub list: &'a Vec<T>,
    pub cursor: &'a Cursor,
    pub focus_style: Style,
}

impl<T: Display> Widget for Select<'_, T> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let capacity = area.height as usize;
        let page_number = self.cursor.current / capacity;
        let idx = self.cursor.current % capacity;

        let render_item = |(i, member): (_, &T)| {
            let content = Text::from(textwrap::wrap(&format!("{member}"), area.width as usize).iter().map(|s| Line::from(s.clone().into_owned())).collect::<Vec<_>>());
            let style = if idx == i && self.focus {
                self.focus_style
            } else {
                Style::default()
            };
            ListItem::new(content).style(style)
        };

        if capacity < self.list.len() {
            let display_items = self
                .list
                .chunks(capacity)
                .nth(page_number)
                .unwrap()
                .iter()
                .enumerate()
                .map(render_item)
                .collect::<Vec<ListItem>>();

            let horizontal_layout = Layout::horizontal([Constraint::Min(3), Constraint::Length(1)]);
            let [list_area, scroll_area] = horizontal_layout.areas(area);
            List::new(display_items).render(list_area, buf);
            CustomScrollBar {
                cursor: self.cursor.current,
                total: self.list.len(),
            }
            .render(scroll_area, buf);
        } else {
            let display_items = self
                .list
                .iter()
                .enumerate()
                .map(render_item)
                .collect::<Vec<ListItem>>();
            List::new(display_items).render(area, buf);
        }
    }
}
