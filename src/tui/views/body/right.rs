use ratatui::{style::Stylize, text::Line, widgets::Widget};

pub struct Right<'a> {
    pub eth_price: &'a Option<String>,
}

impl Widget for Right<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let eth_price = if let Some(eth_price) = self.eth_price {
            eth_price.clone()
        } else {
            "Loading...".to_string()
        };
        Line::from(format!("EthPrice: {eth_price}"))
            .bold()
            .render(area, buf);
    }
}
