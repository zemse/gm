use crate::tui::{controller::navigation::Page, views::components::select::Select};
use ratatui::widgets::Widget;

pub struct Left<'a> {
    pub page: &'a Page,
}

impl Widget for Left<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self.page {
            Page::MainMenu { list, cursor, .. } => Select {
                list,
                cursor: Some(*cursor),
            }
            .render(area, buf),
            _ => unimplemented!(),
        }
    }
}
