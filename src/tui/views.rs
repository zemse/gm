use std::io;

use footer::Footer;
use left::Left;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, BorderType, Widget},
    DefaultTerminal,
};
use right::Right;
use title::Title;

use super::{controller::navigation::Navigation, traits::BorderedWidget};

mod components;
mod footer;
mod left;
mod right;
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
        let horizontal_layout =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]);
        let [left_area, right_area] = horizontal_layout.areas(body_area);

        Title.render(title_area, buf);
        Left {
            page: self.navigation.current_page(),
            text_input: self.navigation.text_input.clone(),
            _marker: std::marker::PhantomData,
        }
        .render_with_block(
            left_area,
            buf,
            Block::bordered().border_type(BorderType::Plain),
        );
        Right {
            eth_price: self.eth_price,
        }
        .render_with_block(
            right_area,
            buf,
            Block::bordered().border_type(BorderType::Plain),
        );
        Footer {
            exit: self.exit,
            navigation: self.navigation,
        }
        .render(footer_area, buf);
    }
}
