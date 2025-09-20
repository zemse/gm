use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};
use std::borrow::Cow;
use textwrap::{wrap, Options};

use crate::extensions::RectExt;

use super::scroll_bar::CustomScrollBar;

#[derive(Default, Debug)]
pub struct TextScroll {
    pub text: String,
    pub scroll_offset: usize,
    pub break_words: bool,
}

impl TextScroll {
    pub fn new(text: String, break_words: bool) -> Self {
        Self {
            text,
            scroll_offset: 0,
            break_words,
        }
    }

    fn lines(&self, width: usize) -> Vec<Cow<'_, str>> {
        self.text
            .lines()
            .flat_map(|line| wrap(line, Options::new(width).break_words(self.break_words)))
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

    pub fn scroll_to_bottom(&mut self, width: usize, height: usize) {
        let lines = self.lines(width).len();
        if lines > height {
            self.scroll_offset = lines - height;
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn get_visible_text(&self, area: Rect) -> (Vec<Cow<'_, str>>, usize) {
        let lines = self.lines(area.width as usize);
        let lines_len = lines.len();
        (
            lines
                .into_iter()
                .skip(self.scroll_offset)
                .take(area.height as usize)
                .collect(),
            lines_len,
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
                total_items: total,
                paginate: false,
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
