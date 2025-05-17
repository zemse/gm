use ratatui::{layout::Offset, style::Stylize, text::Line, widgets::Widget};

pub struct Sidebar<'a> {
    pub online: &'a Option<bool>,
    pub eth_price: &'a Option<String>,
    pub testnet_mode: &'a bool,
}

impl Widget for Sidebar<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let eth_price = if let Some(eth_price) = self.eth_price {
            eth_price.clone()
        } else {
            match self.online {
                Some(true) | None => "Loading...".to_string(),
                Some(false) => "Unable to fetch".to_string(),
            }
        };
        Line::from(format!("EthPrice: {eth_price}"))
            .bold()
            .render(area, buf);
        Line::from(format!("Testnet Mode: {}", self.testnet_mode))
            .bold()
            .render(area.offset(Offset { x: 0, y: 1 }), buf);
    }
}
