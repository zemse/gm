use crate::{
    actions::address_book::AddressBookActions,
    tui::{controller::navigation::Page, views::components::select::Select},
};
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
            Page::AddressBook {
                full_list,
                cursor,
                search_string,
            } => Select {
                list: &full_list
                    .iter()
                    .filter(|item| item.to_string().contains(search_string))
                    .collect::<Vec<&AddressBookActions>>(),
                cursor: Some(*cursor),
            }
            .render(area, buf),
            _ => unimplemented!(),
        }
    }
}
