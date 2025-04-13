use ratatui::{style::Stylize, text::Line, widgets::Widget};

pub struct Left;

impl Widget for Left {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Line::from("left pane content").bold().render(area, buf);
    }
}
