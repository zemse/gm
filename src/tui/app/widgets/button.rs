use crate::tui::theme::Theme;
use crate::tui::traits::BorderedWidget;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::Block,
};

pub struct Button<const REVERSED: bool = false> {
    pub focus: bool,
    pub label: &'static str,
}

impl<const REVERSED: bool> Button<REVERSED> {
    pub fn render(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        theme: &Theme,
    ) where
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
            Block::bordered()
                .border_type(theme.into())
                .style(if self.focus {
                    if REVERSED {
                        Style::default().remove_modifier(Modifier::REVERSED)
                    } else {
                        Style::default().add_modifier(Modifier::REVERSED)
                    }
                } else {
                    Style::default()
                }),
        );
    }
}
