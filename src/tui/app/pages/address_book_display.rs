use crate::{
    disk::{AddressBook, AddressBookEntry, DiskInterface},
    tui::{
        app::{
            widgets::form::{Form, FormItem},
            SharedState,
        },
        events::Event,
        traits::{Component, HandleResult},
    },
};
use ratatui::widgets::Widget;
use std::sync::{atomic::AtomicBool, mpsc, Arc};

pub struct AddressBookDisplayPage {
    pub id: usize,
    pub form: Form,
}

impl AddressBookDisplayPage {
    pub fn new(id: usize, name: String, address: String) -> Self {
        Self {
            id,
            form: Form {
                cursor: 0,
                items: vec![
                    FormItem::Heading("Create New AddressBook entry"),
                    FormItem::InputBox {
                        label: "name",
                        text: name,
                        empty_text: None,
                    },
                    FormItem::InputBox {
                        label: "address",
                        text: address,
                        empty_text: None,
                    },
                    FormItem::Button { label: "Save" },
                    FormItem::ErrorText(String::new()),
                ],
            },
        }
    }
}

impl Component for AddressBookDisplayPage {
    fn handle_event(
        &mut self,
        event: &Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
    ) -> crate::Result<HandleResult> {
        let mut handle_result = HandleResult::default();

        self.form.handle_event(event, |label, form| {
            if label == "Save" {
                let name = form.get_input_text(1);
                if name.is_empty() {
                    let error = form.get_error_text_mut(4);
                    *error = "Please enter name, you cannot leave it empty".to_string();
                } else {
                    let mut address_book = AddressBook::load();

                    let address = form.get_input_text(2);
                    let result = address.parse().map_err(crate::Error::from).map(|address| {
                        address_book.update(
                            self.id,
                            AddressBookEntry {
                                name: name.clone(),
                                address,
                            },
                        );
                    });
                    if let Err(e) = result {
                        let error = form.get_error_text_mut(4);
                        *error = format!("{e:?}");
                    } else {
                        handle_result.page_pops = 1;
                        handle_result.reload = true;
                    }
                }
            }
        })?;

        Ok(handle_result)
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        _: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf);
        area
    }
}
