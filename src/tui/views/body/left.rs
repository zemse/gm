use std::marker::PhantomData;

use crate::tui::{
    controller::navigation::Page,
    views::components::{filter_select::FilterSelect, select::Select},
};
use ratatui::widgets::Widget;

pub struct Left<'a> {
    pub page: &'a Page,
    pub text_input: Option<String>,
    pub _marker: PhantomData<&'a ()>,
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

            Page::AddressBook { full_list, cursor } => FilterSelect {
                full_list,
                cursor: Some(*cursor),
                search_string: &self.text_input.unwrap_or_default(),
            }
            .render(area, buf),

            _ => unimplemented!(),
        }
    }
}
