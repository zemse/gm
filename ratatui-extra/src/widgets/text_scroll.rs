use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

use crate::extensions::RectExt;

use super::scroll_bar::CustomScrollBar;

#[derive(Default)]
pub struct TextScroll {
    pub text: String,
    pub scroll_offset: usize,
}

impl TextScroll {
    pub fn new(text: String) -> Self {
        Self {
            text,
            scroll_offset: 0,
        }
    }

    fn lines(&self, width: usize) -> Vec<&str> {
        self.text
            .lines()
            .flat_map(|line| split_str_by_width(line, width))
            .collect()
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self, width: usize, height: usize) {
        let lines = self.lines(width).len();
        if self.scroll_offset + height < lines {
            self.scroll_offset += 1;
        }
    }

    pub fn get_visible_text(&self, area: Rect) -> (Vec<&str>, usize) {
        let lines: Vec<&str> = self.lines(area.width as usize);
        (
            lines
                .iter()
                .skip(self.scroll_offset)
                .take(area.height as usize)
                .map(|line| line.trim_end())
                .collect(),
            lines.len(),
        )
    }

    pub fn handle_event(&mut self, key_event: Option<&KeyEvent>, area: ratatui::prelude::Rect) {
        if let Some(key_event) = key_event {
            match key_event.code {
                KeyCode::Up => {
                    self.scroll_up();
                }
                KeyCode::Down => {
                    self.scroll_down(area.width as usize, area.height as usize);
                }
                _ => {}
            }
        }
    }
}

fn split_str_by_width(s: &str, width: usize) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut count = 0;

    for (i, _) in s.char_indices() {
        if count == width {
            result.push(&s[start..i]);
            start = i;
            count = 0;
        }
        count += 1;
    }

    if start < s.len() {
        result.push(&s[start..]);
    }

    result
}

impl Widget for &TextScroll {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [mut text_area, scroll_area] =
            Layout::horizontal([Constraint::Min(1), Constraint::Length(1)]).areas(area);

        let (lines, total) = self.get_visible_text(text_area);
        if total > area.height as usize {
            for line in &lines {
                line.render(text_area, buf);
                let Some(text_area_new) = text_area.consume_height(1) else {
                    return;
                };
                text_area = text_area_new;
            }

            CustomScrollBar {
                cursor: self.scroll_offset,
                total: total - area.height as usize,
                paginate: true,
            }
            .render(scroll_area, buf);
        } else {
            for line in &lines {
                line.render(text_area, buf);
                let Some(text_area_new) = text_area.consume_height(1) else {
                    return;
                };
                text_area = text_area_new;
            }
        }
    }
}
