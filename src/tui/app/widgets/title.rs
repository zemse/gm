use alloy::primitives::Address;
use ratatui::{layout::Rect, style::Stylize, text::Line, widgets::Widget};

pub struct Title<'a> {
    pub current_account: Option<&'a Address>,
}

impl Widget for Title<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let pkg_version = env!("CARGO_PKG_VERSION");

        Line::from(format!("version {pkg_version}"))
            .bold()
            .right_aligned()
            .render(
                Rect {
                    x: area.x + 1,
                    y: area.y,
                    width: area.width - 2,
                    height: area.height,
                },
                buf,
            );

        let welcome_string = if let Some(account) = self.current_account {
            format!("gm {}", account)
        } else {
            "gm wallet".to_string()
        };

        Line::from(welcome_string).bold().render(
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
