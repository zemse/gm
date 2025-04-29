use ratatui::{
    layout::{Margin, Offset, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Widget},
};

pub struct PopupOk<'a> {
    pub title: &'a str,
    pub message: &'a String,
}

impl Widget for PopupOk<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let popup_area = if area.width > 4 && area.height > 4 {
            Rect {
                width: area.width - 4,
                height: area.height - 4,
                x: area.x + 2,
                y: area.y + 2,
            }
        } else {
            area
        };

        Clear.render(popup_area, buf);
        Block::default()
            .style(Style::default().bg(Color::Red))
            .render(popup_area, buf);

        let popup_inner_area = popup_area.inner(Margin::new(2, 1));

        let block = Block::bordered();
        let display_area = block.inner(popup_inner_area);
        block.render(popup_inner_area, buf);

        Line::from(self.title).bold().render(display_area, buf);
        self.message
            .render(display_area.offset(Offset { x: 0, y: 2 }), buf);
    }
}
