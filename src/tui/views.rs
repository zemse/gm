use std::io;

use body::Body;
use footer::Footer;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Widget,
    DefaultTerminal,
};
use title::Title;

use super::{controller::navigation::Navigation, traits::BorderedWidget};

mod body;
mod components;
mod footer;
mod title;

pub struct View<'a> {
    pub exit: bool,
    pub eth_price: &'a Option<String>,
    pub navigation: &'a Navigation<'a>,
}

impl View<'_> {
    pub fn draw(&self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| {
            frame.render_widget(
                &View {
                    exit: self.exit,
                    eth_price: self.eth_price,
                    navigation: self.navigation,
                },
                frame.area(),
            );
        })?;
        Ok(())
    }
}

impl Widget for &View<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vertical_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ]);
        let [title_area, body_area, footer_area] = vertical_layout.areas(area);

        Title.render(title_area, buf);
        Body {
            eth_price: self.eth_price,
            navigation: self.navigation,
        }
        .render(body_area, buf);
        Footer {
            exit: self.exit,
            navigation: self.navigation,
        }
        .render(footer_area, buf);
    }
}
