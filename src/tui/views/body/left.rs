use crate::tui::controller::navigation::Navigation;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{List, ListItem, Widget},
};

pub struct Left<'a> {
    pub cursor: &'a Navigation,
}

impl Widget for Left<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let idx = self.cursor.left_idx();
        let items: Vec<ListItem> = self
            .cursor
            .left_list()
            .into_iter()
            .enumerate()
            .map(|(i, action)| {
                let content = Line::from(action);
                let style = if Some(i) == idx {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(content).style(style)
            })
            .collect();

        List::new(items).render(area, buf);
    }
}
