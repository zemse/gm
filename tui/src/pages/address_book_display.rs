use crate::{
    app::SharedState, post_handle_event::PostHandleEventActions, traits::Component, AppEvent,
};
use gm_ratatui_extra::{
    act::Act,
    button::Button,
    form::{Form, FormItemIndex, FormWidget},
    input_box_owned::InputBoxOwned,
};
use gm_utils::{
    address_book::{AddressBookEntry, AddressBookStore},
    alloy::StringExt,
    disk_storage::DiskStorageInterface,
};
use ratatui::{buffer::Buffer, layout::Rect};
use std::{borrow::Cow, sync::mpsc};
use strum::{Display, EnumIter};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Display, EnumIter, PartialEq)]
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
                widget: InputBoxOwned::new("Name"),
            },
            FormItem::Address => FormWidget::InputBox {
                widget: InputBoxOwned::new("Address"),
            },
            FormItem::SaveButton => FormWidget::Button {
                widget: Button::new("Save"),
            },
            FormItem::ErrorText => FormWidget::ErrorText(String::new()),
        };
        Ok(widget)
    }
}

#[derive(Debug)]
pub struct AddressBookDisplayPage {
    pub id: usize,
    pub form: Form<FormItem, crate::Error>,
}

impl AddressBookDisplayPage {
    pub fn new(id: usize, name: String, address: String) -> crate::Result<Self> {
        Ok(Self {
            id,
            form: Form::init(|form| {
                form.set_text(FormItem::Name, name);
                form.set_text(FormItem::Address, address);
                Ok(())
            })?,
        })
    }
}

impl Component for AddressBookDisplayPage {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Display")
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _popup_area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let mut handle_result = PostHandleEventActions::default();

        let r = self.form.handle_event(
            event.widget_event().as_ref(),
            area,
            |_, _| Ok(()),
            |label, form| {
                if label == FormItem::SaveButton {
                    let name = form.get_text(FormItem::Name);
                    if name.is_empty() {
                        form.set_text(
                            FormItem::ErrorText,
                            "Please enter name, you cannot leave it empty".to_string(),
                        );
                    } else {
                        let mut address_book = AddressBookStore::load()?;

                        let address = form.get_text(FormItem::Address);
                        let result = address.parse_as_address().and_then(|address| {
                            address_book.update(
                                self.id,
                                AddressBookEntry {
                                    name: name.to_string(),
                                    address,
                                },
                            )
                        });
                        if let Err(e) = result {
                            form.set_text(FormItem::ErrorText, format!("{e:?}"));
                        } else {
                            handle_result.page_pop();
                            handle_result.reload();
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
