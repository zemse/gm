use ratatui::{
    layout::{Margin, Rect},
    style::{Modifier, Style},
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
}

impl Widget for Popup {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let popup_area = Popup::area(area);
        // let popup_inner_area = Popup::inner_area(area);

        Clear.render(popup_area, buf);

        Block::default()
            .style(Style::default().add_modifier(Modifier::REVERSED))
            .render(popup_area, buf);
    }
}
