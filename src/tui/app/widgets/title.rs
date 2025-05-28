use alloy::primitives::Address;
use ratatui::{style::Stylize, text::Line, widgets::Widget};

use crate::tui::traits::RectUtil;

pub struct Title<'a> {
    pub current_account: Option<&'a Address>,
    pub online: Option<bool>,
}

impl Widget for Title<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let area = area.margin_h(1);

        let welcome_string = format!(
            "gm {account}",
            account = self
                .current_account
                .map(|a| a.to_string())
                .unwrap_or("wallet".to_string())
        );

        Line::from(welcome_string).bold().render(area, buf);

        let pkg_version = env!("CARGO_PKG_VERSION");
        Line::from(format!(
            "version {pkg_version}{}",
            match self.online {
                Some(true) => " - online",
                Some(false) => " - offline",
                None => "",
            }
        ))
        .bold()
        .right_aligned()
        .render(area, buf);
    }
}
