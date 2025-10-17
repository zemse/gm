use std::time::{Duration, Instant};

use gm_utils::text_wrap::text_wrap;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    text::Span,
    widgets::{Block, Clear, Widget},
};

use crate::{event::WidgetEvent, extensions::RectExt, thematize::Thematize};

pub struct Toast {
    shown: bool,
    message: &'static str,
    max_width: Option<usize>,
    position: Position,
    expiry_instant: Instant,
}

impl Toast {
    pub fn new(message: &'static str) -> Self {
        Self {
            shown: false,
            message,
            max_width: None,
            position: Position::default(),
            expiry_instant: Instant::now(),
        }
    }

    pub fn show(&mut self, position: Position, duration: Duration) {
        self.shown = true;
        self.position = position;
        self.expiry_instant = Instant::now() + duration;
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expiry_instant
    }

    pub fn handle_event(&mut self, _event: Option<WidgetEvent>) {
        if self.is_expired() {
            self.shown = false;
        }
    }

    pub fn render(&self, buf: &mut Buffer, theme: &impl Thematize) {
        let area = buf.area();

        if self.shown {
            let width = if let Some(max_width) = self.max_width {
                (self.message.len()).min(max_width)
            } else {
                self.message.len()
            } as u16;

            // Render at specified position
            let width = width.min(area.width - self.position.x);
            let lines = text_wrap(self.message, width);

            let clear_area = Rect {
                x: self.position.x,
                y: self.position.y + 1,
                width: width + 4,
                height: lines.len() as u16 + 2,
            };
            let text_area = clear_area.block_inner().margin_left(1).margin_right(1);

            Clear.render(clear_area, buf);
            Block::default()
                .style(theme.toast())
                .render(clear_area, buf);
            Span::raw(self.message)
                .style(theme.toast())
                .render(text_area, buf);
        }
    }
}
