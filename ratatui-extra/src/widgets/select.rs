use std::fmt::Display;

use super::cursor::Cursor;
use super::scroll_bar::CustomScrollBar;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    text::Line,
    widgets::{List, ListItem, Widget},
};
use textwrap::{Options, WrapAlgorithm};

pub struct Select<'a, T: Display> {
    pub focus: bool,
    pub list: &'a Vec<T>,
    pub cursor: &'a Cursor,
    pub focus_style: Style,
}

impl<T: Display> Select<'_, T> {
    pub fn display_item(
        &self,
        area: Rect,
    ) -> (Vec<ListItem<'_>>, Vec<usize>, usize, usize, usize, usize) {
        let capacity = area.height as usize;
        let horizontal_layout = Layout::horizontal([Constraint::Min(3), Constraint::Length(1)]);
        let [list_area, _] = horizontal_layout.areas(area);

        let mut list_height = 0;
        let mut temp_rows = 0;
        let mut temp_rows_2 = 0;
        let mut start_index = 0;
        let mut prev_start_index = 0;
        let mut end_index = self.list.len() - 1;
        let mut found = false;
        let mut item_heights = vec![];

        let mut current_page: usize = 0;
        let mut total_pages: usize = 1;
        let render_item = |(i, member): (usize, &T)| {
            let text = &format!("{member}");
            let wrapped_text = textwrap::wrap(
                text,
                Options::new(if capacity < list_height {
                    list_area.width as usize
                } else {
                    area.width as usize
                })
                .wrap_algorithm(WrapAlgorithm::FirstFit),
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
                if self.cursor.current <= prev_index && self.cursor.current >= start_index {
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
        (
            render_items[start_index..=end_index].to_vec(),
            item_heights[start_index..=end_index].to_vec(),
            start_index,
            list_height,
            current_page,
            total_pages,
        )
    }

    pub fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if self.list.is_empty() {
            return;
        }

        let capacity = area.height as usize;
        let horizontal_layout = Layout::horizontal([Constraint::Min(3), Constraint::Length(1)]);
        let [list_area, scroll_area] = horizontal_layout.areas(area);

        let (display_items, _, _, list_height, current_page, total_pages) = self.display_item(area);

        if capacity < list_height {
            List::new(display_items).render(list_area, buf);
            CustomScrollBar {
                cursor: current_page,
                total_items: total_pages,
                paginate: false,
            }
            .render(scroll_area, buf);
        } else {
            List::new(display_items).render(area, buf);
        }
    }
}
