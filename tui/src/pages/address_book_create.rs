use crate::{
    app::SharedState,
    events::Event,
    traits::{Actions, Component},
};
use gm_ratatui_extra::{
    act::Act,
    form::{Form, FormItemIndex, FormWidget},
};
use gm_utils::{
    address_book::{AddressBookEntry, AddressBookStore},
    disk_storage::DiskStorageInterface,
};
use std::sync::{atomic::AtomicBool, mpsc, Arc};
use strum::{Display, EnumIter};

#[derive(EnumIter, PartialEq, Display)]
pub enum FormItem {
    Heading,
    Name,
    Address,
    SaveButton,
    ErrorText,
}
impl FormItemIndex for FormItem {
    fn index(self) -> usize {
        self as usize
    }
}
impl TryFrom<FormItem> for FormWidget {
    type Error = crate::Error;
    fn try_from(value: FormItem) -> crate::Result<Self> {
        let widget = match value {
            FormItem::Heading => FormWidget::Heading("Edit AddressBook entry"),
            FormItem::Name => FormWidget::InputBox {
                label: "name",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::Address => FormWidget::InputBox {
                label: "address",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::SaveButton => FormWidget::Button { label: "Save" },
            FormItem::ErrorText => FormWidget::ErrorText(String::new()),
        };
        Ok(widget)
    }
}

pub struct AddressBookCreatePage {
    pub form: Form<FormItem, crate::Error>,
}

impl AddressBookCreatePage {
    pub fn new(name: String, address: String) -> crate::Result<Self> {
        Ok(Self {
            form: Form::init(|form| {
                *form.get_text_mut(FormItem::Name) = name;
                *form.get_text_mut(FormItem::Address) = address;
                Ok(())
            })?,
        })
    }
}

impl Component for AddressBookCreatePage {
    fn handle_event(
        &mut self,
        event: &Event,
        _area: ratatui::prelude::Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let mut handle_result = Actions::default();

        let r = self.form.handle_event(
            event.key_event(),
            |_, _| Ok(()),
            |label, form| {
                if label == FormItem::SaveButton {
                    let name = form.get_text(FormItem::Name);
                    if name.is_empty() {
                        let error = form.get_text_mut(FormItem::ErrorText);
                        *error = "Please enter name, you cannot leave it empty".to_string();
                    } else {
                        let mut address_book = AddressBookStore::load()?;

                        let address = form.get_text(FormItem::Address);

                        let result = address
                            .parse()
                            .map_err(|_| crate::Error::InvalidAddress(address.clone()))
                            .and_then(|address| {
                                address_book
                                    .add(AddressBookEntry {
                                        name: name.clone(),
                                        address,
                                    })
                                    .map_err(crate::Error::from)
                            });
                        if let Err(e) = result {
                            let error = form.get_text_mut(FormItem::ErrorText);
                            *error = format!("{e:?}");
                        } else {
                            handle_result.page_pops = 1;
                            handle_result.reload = true;
                        }
                    }
                }
                Ok(())
            },
        )?;
        handle_result.merge(r);

        Ok(handle_result)
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        ss: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf, &ss.theme);

        area
    }
}
