use std::marker::PhantomData;

use crate::tui::{
    controller::navigation::Page,
    views::components::{filter_select::FilterSelect, select::Select},
};
use ratatui::widgets::Widget;

use super::components::form::{Form, FormItem};

pub struct Left<'a> {
    pub page: Option<&'a Page>,
    pub _marker: PhantomData<&'a ()>,
}

impl Widget for Left<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if let Some(page) = self.page {
            match page {
                Page::MainMenu { list, cursor, .. } => Select {
                    list,
                    cursor: Some(*cursor),
                }
                .render(area, buf),

                Page::AddressBook {
                    full_list,
                    cursor,
                    search_string,
                } => FilterSelect {
                    full_list,
                    cursor: Some(*cursor),
                    search_string,
                }
                .render(area, buf),

                Page::AddressBookCreateNewEntry {
                    cursor,
                    name,
                    address,
                    error,
                } => Form {
                    items: vec![
                        FormItem::Heading("Create New AddressBook entry"),
                        FormItem::InputBox {
                            focus: *cursor == 0,
                            label: &"name".to_string(),
                            text: name,
                        },
                        FormItem::InputBox {
                            focus: *cursor == 1,
                            label: &"address".to_string(),
                            text: address,
                        },
                        FormItem::Button {
                            focus: *cursor == 2,
                            label: &"Save".to_string(),
                        },
                        FormItem::Error {
                            label: &error.as_ref(),
                        },
                    ],
                }
                .render(area, buf),

                Page::AddressBookDisplayEntry {
                    cursor,
                    name,
                    address,
                    error,
                    ..
                } => Form {
                    items: vec![
                        FormItem::Heading("Edit AddressBook entry"),
                        FormItem::InputBox {
                            focus: *cursor == 0,
                            label: &"name".to_string(),
                            text: name,
                        },
                        FormItem::InputBox {
                            focus: *cursor == 1,
                            label: &"address".to_string(),
                            text: address,
                        },
                        FormItem::Button {
                            focus: *cursor == 2,
                            label: &"Save".to_string(),
                        },
                        FormItem::Error {
                            label: &error.as_ref(),
                        },
                    ],
                }
                .render(area, buf),
            }
        }
    }
}
