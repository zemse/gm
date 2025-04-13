use ratatui::{style::Stylize, text::Line, widgets::Widget};

pub struct Right<'a> {
    pub eth_price: &'a Option<String>,
}

impl Widget for Right<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Line::from(format!(
            "EthPrice: {}",
            self.eth_price.as_ref().unwrap_or(&"Loading...".to_string())
        ))
        .bold()
        .render(area, buf);
    }
}
