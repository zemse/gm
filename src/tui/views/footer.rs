use ratatui::{layout::Rect, text::Line, widgets::Widget};

use crate::tui::controller::navigation::Navigation;

pub struct Footer<'a> {
    pub exit: bool,
    pub navigation: &'a Navigation<'a>,
}

impl Widget for Footer<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let footer_text = if self.exit {
            "exiting please wait..."
        } else if self.navigation.is_main_menu() {
            "press control c or [ESC] to exit"
        // } else if self.navigation.is_text_input_user_typing() {
        //     "press control c to quit | press [ESC] to clear text input"
        } else {
            "press control c to quit | press [ESC] to go back"
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
