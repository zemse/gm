use ratatui::{layout::Rect, style::Stylize, text::Line, widgets::Widget};

pub struct Title;

impl Widget for Title {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Line::from(format!("gm wallet v{}", env!("CARGO_PKG_VERSION")))
            .bold()
            .render(
                Rect {
                    x: area.x + 1,
                    y: area.y,
                    width: area.width - 2,
                    height: area.height,
                },
                buf,
            );
    }
}
