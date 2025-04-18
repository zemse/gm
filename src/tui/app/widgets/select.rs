use std::fmt::Display;

use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{List, ListItem, Widget},
};

pub struct Select<'a, T: Display> {
    pub list: &'a Vec<T>,
    pub cursor: Option<&'a usize>,
}

impl<T: Display> Widget for Select<'_, T> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let idx = self.cursor;
        let items: Vec<ListItem> = self
            .list
            .iter()
            .enumerate()
            .map(|(i, member)| {
                let content = Line::from(format!("{member}"));
                let style = if idx == Some(&i) {
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
