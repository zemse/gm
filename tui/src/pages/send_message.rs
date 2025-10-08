use crate::app::SharedState;
use crate::pages::tx_popup::TxPopup;
use crate::post_handle_event::PostHandleEventActions;
use crate::traits::Component;
use crate::widgets::{address_book_popup, networks_popup, AddressBookPopup, NetworksPopup};
use gm_ratatui_extra::act::Act;
use gm_ratatui_extra::button::Button;
use gm_ratatui_extra::form::{Form, FormWidget};
use gm_ratatui_extra::input_box_owned::InputBoxOwned;
use gm_ratatui_extra::thematize::Thematize;
use gm_ratatui_extra::widgets::form::FormItemIndex;
use gm_utils::alloy::StringExt;
use gm_utils::disk_storage::DiskStorageInterface;
use gm_utils::network::{Network, NetworkStore};
use ratatui::buffer::Buffer;
use tokio_util::sync::CancellationToken;

use super::address_book::AddressBookMenuItem;
use crate::{AppEvent, Result};

use alloy::primitives::Bytes;
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use ratatui::layout::Rect;
use std::sync::mpsc;
use strum::{Display, EnumIter};

#[derive(Debug, Display, EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    To,
    Message,
    Network,
    SendMessageButton,
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
            FormItem::Heading => FormWidget::Heading("Send a Message"),
            FormItem::To => FormWidget::InputBox {
                widget: InputBoxOwned::new("To")
                    .with_empty_text("press SPACE to select from address book"),
            },
            FormItem::Message => FormWidget::InputBox {
                widget: InputBoxOwned::new("Message").with_empty_text("Type message to send"),
            },
            FormItem::Network => FormWidget::DisplayBox {
                widget: InputBoxOwned::new("Network")
                    .with_empty_text("press SPACE to select network"),
            },
            FormItem::SendMessageButton => FormWidget::Button {
                widget: Button::new("Send Message"),
            },
        };
        Ok(widget)
    }
}

#[derive(Debug)]
pub struct SendMessagePage {
    pub form: Form<FormItem, crate::Error>,
    pub address_book_popup: AddressBookPopup,
    pub networks_popup: NetworksPopup,
    pub tx_popup: TxPopup,
}

impl SendMessagePage {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            form: Form::init(|_| Ok(()))?,
            address_book_popup: address_book_popup(),
            networks_popup: networks_popup(),
            tx_popup: TxPopup::default(),
        })
    }
}

impl Component for SendMessagePage {
    fn set_focus(&mut self, focus: bool) {
        self.form.set_form_focus(focus);
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _popup_area: Rect,
        tr: &mpsc::Sender<AppEvent>,
        sd: &CancellationToken,
        ss: &SharedState,
    ) -> Result<PostHandleEventActions> {
        let mut result = PostHandleEventActions::default();

        #[allow(clippy::single_match)]
        match event {
            AppEvent::ConfigUpdate => {
                let network_name = self.form.get_text(FormItem::Network);
                let network_store = NetworkStore::load()?;
                let network = network_store
                    .get_by_name(&network_name)
                    .ok_or(crate::Error::NetworkNotFound(network_name.to_string()))?;

                if network.is_testnet != ss.testnet_mode {
                    self.form.set_text(FormItem::Network, "".to_string());
                }
            }
            _ => {}
        }

        if self.address_book_popup.is_open() {
            result.merge(self.address_book_popup.handle_event(
                event.key_event(),
                |entry| -> crate::Result<()> {
                    self.form
                        .set_text(FormItem::To, entry.address()?.to_string());

                    self.form.advance_cursor();
                    Ok(())
                },
            )?);
        } else if self.networks_popup.is_open() {
            result.merge(self.networks_popup.handle_event(
                event.key_event(),
                |network| -> crate::Result<()> {
                    self.form.set_text(FormItem::Network, network.name.clone());

                    self.form.advance_cursor();
                    Ok(())
                },
            )?);
        } else if self.tx_popup.is_open() {
            let r = self.tx_popup.handle_event(
                (event, area, tr, sd, ss),
                |_| Ok(()),
                |_| Ok(()),
                |_, _, _| Ok(()),
                || Ok(()),
                || Ok(()),
            )?;
            result.merge(r);
        } else {
            // Handle form events
            if self.form.is_focused(FormItem::To)
                && self.form.get_text(FormItem::To).is_empty()
                && event.is_space_or_enter_pressed()
            {
                self.address_book_popup.open();
                self.address_book_popup
                    .set_items(Some(AddressBookMenuItem::get_menu(
                        false,
                        ss.recent_addresses.clone(),
                    )?));
            } else if self.form.is_focused(FormItem::Network) && event.is_space_or_enter_pressed() {
                self.networks_popup.open();
                self.networks_popup
                    .set_items(Some(NetworkStore::load()?.filter(ss.testnet_mode)));
            } else {
                let r = self.form.handle_event(
                    event.widget_event().as_ref(),
                    area,
                    |_, _| Ok(()),
                    |label, form| {
                        if label == FormItem::SendMessageButton {
                            let to = form.get_text(FormItem::To);
                            let message = form.get_text(FormItem::Message);
                            let network_name = form.get_text(FormItem::Network);
                            if message.is_empty() {
                                return Err(crate::Error::CannotBeEmpty("Message".to_string()));
                            }

                            self.tx_popup.set_tx_req(
                                Network::from_name(&network_name)?,
                                TransactionRequest::default()
                                    .to(to.parse_as_address()?)
                                    .input(TransactionInput::from(Bytes::from(
                                        message.to_string().into_bytes(),
                                    ))),
                            );
                            self.tx_popup.open();
                        }

                        Ok(())
                    },
                )?;
                result.merge(r);
            }
        }

        Ok(result)
    }

    fn render_component(
        &self,
        area: Rect,
        popup_area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, popup_area, buf, &shared_state.theme);

        self.address_book_popup
            .render(popup_area, buf, &shared_state.theme.popup());

        self.networks_popup
            .render(popup_area, buf, &shared_state.theme.popup());

        self.tx_popup
            .render(popup_area, buf, &shared_state.theme.popup());

        area
    }
}
