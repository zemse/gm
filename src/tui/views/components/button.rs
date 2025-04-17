use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Widget},
};

use crate::tui::traits::BorderedWidget;

pub struct Button<'a> {
    pub focus: bool,
    pub label: &'a String,
}

impl Widget for Button<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let button_area = Rect {
            width: (self.label.len() + 2) as u16,
            height: 3,
            x: area.x,
            y: area.y,
        };

        Line::from(self.label.clone()).render_with_block(
            button_area,
            buf,
            Block::bordered().style(if self.focus {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            }),
        );
    }
}
