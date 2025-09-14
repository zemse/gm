use crate::thematize::Thematize;
use ratatui::{
    layout::{Margin, Rect},
    widgets::{Block, Clear, Widget},
};

pub struct Popup;

impl Popup {
    pub fn area(full_area: Rect) -> Rect {
        if full_area.width > 4 && full_area.height > 4 {
            Rect {
                width: full_area.width - 4,
                height: full_area.height - 4,
                x: full_area.x + 2,
                y: full_area.y + 2,
            }
        } else {
            full_area
        }
    }

    pub fn inner_area(full_area: Rect) -> Rect {
        Self::area(full_area).inner(Margin::new(2, 1))
    }

    pub fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        theme: &impl Thematize,
    ) where
        Self: Sized,
    {
        let theme = theme.popup();
        let popup_area = Popup::area(area);

        Clear.render(popup_area, buf);

        Block::default()
            .border_type(theme.border_type())
            .style(theme.block())
            .render(popup_area, buf);
    }
}
