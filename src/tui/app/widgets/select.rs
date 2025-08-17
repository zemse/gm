use std::fmt::Display;

use crate::utils::cursor::Cursor;
use ratatui::text::Text;
use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    text::Line,
    widgets::{List, ListItem, Widget},
};

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
        let horizontal_layout = Layout::horizontal([Constraint::Min(3), Constraint::Length(1)]);
        let [list_area, scroll_area] = horizontal_layout.areas(area);

        let mut list_height = 0;
        let mut temp_rows = 0;
        let mut start_index = 0;
        let mut prev_start_index = 0;
        let mut end_index = self.list.len() - 1;
        let mut found = false;
        let render_item = |(i, member): (usize, &T)| {
            let text = &format!("{member}");
            let wrapped_text = textwrap::wrap(
                text,
                if capacity < self.list.len() {
                    list_area.width as usize
                } else {
                    area.width as usize
                },
            );
            list_height += wrapped_text.len();
            if !found {
                temp_rows += wrapped_text.len();
            }

            if temp_rows > capacity {
                temp_rows = wrapped_text.len();
                let prev_index: usize = i.saturating_sub(1);
                if self.cursor.current <= prev_index && self.cursor.current >= start_index {
                    end_index = prev_index;
                    prev_start_index = start_index;
                    found = true;
                } else {
                    start_index = i;
                }
            }
            let content = Text::from(
                wrapped_text
                    .iter()
                    .map(|s| Line::from(s.clone().into_owned()))
                    .collect::<Vec<_>>(),
            );
            let style = if self.cursor.current == i && self.focus {
                self.focus_style
            } else {
                Style::default()
            };
            ListItem::new(content).style(style)
        };
        let render_items = self
            .list
            .iter()
            .enumerate()
            .map(render_item)
            .collect::<Vec<ListItem>>();
        let display_items = render_items[start_index..=end_index].to_vec();

        if capacity < list_height {
            List::new(display_items).render(list_area, buf);
            CustomScrollBar {
                cursor: self.cursor.current,
                total: self.list.len(),
            }
            .render(scroll_area, buf);
        } else {
            List::new(display_items).render(area, buf);
        }
    }
}
