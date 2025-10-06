use crate::thematize::Thematize;
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    widgets::{Block, Clear, Widget},
};

pub struct Popup;

impl Popup {
    pub fn inner_area(full_area: Rect) -> Rect {
        full_area.inner(Margin::new(2, 1))
    }

    pub fn render(self, popup_area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        let theme = theme.popup();

        Clear.render(popup_area, buf);

        Block::default()
            .border_type(theme.border_type())
            .style(theme.style())
            .render(popup_area, buf);
    }
}
