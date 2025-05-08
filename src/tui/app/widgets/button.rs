use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Widget},
};

use crate::tui::traits::BorderedWidget;

pub struct Button {
    pub focus: bool,
    pub label: &'static str,
}

impl Widget for Button {
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

        Line::from(self.label).render_with_block(
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
