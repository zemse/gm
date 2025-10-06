use crate::{
    app::SharedState,
    traits::{Actions, Component},
    AppEvent,
};
use gm_ratatui_extra::{
    act::Act,
    form::{Form, FormItemIndex, FormWidget},
};
use gm_utils::{
    address_book::{AddressBookEntry, AddressBookStore},
    alloy::StringExt,
    disk_storage::DiskStorageInterface,
};
use ratatui::{buffer::Buffer, layout::Rect};
use std::sync::mpsc;
use strum::{Display, EnumIter};
use tokio_util::sync::CancellationToken;

#[derive(Debug, EnumIter, PartialEq, Display)]
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
            FormItem::SaveButton => FormWidget::Button {
                label: "Save",
                hover_focus: false,
            },
            FormItem::ErrorText => FormWidget::ErrorText(String::new()),
        };
        Ok(widget)
    }
}

#[derive(Debug)]
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
        event: &AppEvent,
        area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let mut handle_result = Actions::default();

        let r = self.form.handle_event(
            event.input_event(),
            area,
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

                        let result = address.parse_as_address().and_then(|address| {
                            address_book.add(AddressBookEntry {
                                name: name.clone(),
                                address,
                            })
                        });

                        if let Err(e) = result {
                            let error = form.get_text_mut(FormItem::ErrorText);
                            *error = format!("{e:?}");
                        } else {
                            handle_result.page_pop = true;
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
        area: Rect,
        popup_area: Rect,
        buf: &mut Buffer,
        ss: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, popup_area, buf, &ss.theme);

        area
    }
}
