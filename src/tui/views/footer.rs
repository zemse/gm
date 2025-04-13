use ratatui::{layout::Rect, text::Line, widgets::Widget};

pub struct Footer {
    pub exit: bool,
}

impl Widget for Footer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let footer_text = if self.exit {
            "exiting please wait..."
        } else {
            "press 'q' to exit"
        };
        Line::from(footer_text).render(
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
