use crate::disk::DiskInterface;
use crate::network::NetworkStore;
use crate::tui::app::widgets::address_book_popup::AddressBookPopup;
use crate::tui::app::widgets::form::FormItemIndex;
use crate::tui::app::widgets::networks_popup::NetworksPopup;
use crate::tui::app::widgets::tx_popup::TxPopup;
use crate::tui::app::SharedState;
use crate::tui::{
    app::widgets::form::{Form, FormWidget}, // <- Using your custom form system
    events::Event,
    traits::{Component, HandleResult},
};

use super::address_book::AddressBookMenuItem;
use crate::Result;

use alloy::primitives::Bytes;
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use ratatui::layout::Rect;
use std::sync::mpsc;
use std::sync::{atomic::AtomicBool, Arc};
use strum::EnumIter;

#[derive(EnumIter, PartialEq)]
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
                label: "To",
                text: String::new(),
                empty_text: Some("<press SPACE to select from address book>"),
                currency: None,
            },
            FormItem::Message => FormWidget::InputBox {
                label: "Message",
                text: String::new(),
                empty_text: Some("Type message to send"),
                currency: None,
            },
            FormItem::Network => FormWidget::DisplayBox {
                label: "Network",
                text: String::new(),
                empty_text: Some("<press SPACE to select network>"),
            },
            FormItem::SendMessageButton => FormWidget::Button {
                label: "Send Message",
            },
        };
        Ok(widget)
    }
}

pub struct SendMessagePage {
    pub form: Form<FormItem>,
    pub address_book_popup: AddressBookPopup,
    pub networks_popup: NetworksPopup,
    pub tx_popup: TxPopup,
}

impl SendMessagePage {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            form: Form::init(|_| Ok(()))?,
            address_book_popup: AddressBookPopup::default(),
            networks_popup: NetworksPopup::default(),
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
        event: &Event,
        area: Rect,
        tr: &mpsc::Sender<Event>,
        sd: &Arc<AtomicBool>,
        ss: &SharedState,
    ) -> Result<HandleResult> {
        let mut result = HandleResult::default();

        #[allow(clippy::single_match)]
        match event {
            Event::ConfigUpdate => {
                let network_name = self.form.get_text(FormItem::Network);
                let network_store = NetworkStore::load()?;
                let network = network_store
                    .get_by_name(network_name)
                    .ok_or(crate::Error::NetworkNotFound(network_name.to_string()))?;

                if network.is_testnet != ss.testnet_mode {
                    self.form.get_text_mut(FormItem::Network).clear();
                }
            }
            _ => {}
        }

        if self.address_book_popup.is_open() {
            result.merge(self.address_book_popup.handle_event(event, |entry| {
                let to_address = self.form.get_text_mut(FormItem::To);
                *to_address = entry.address_unwrap().to_string();
                self.form.advance_cursor();
            })?);
        } else if self.networks_popup.is_open() {
            result.merge(self.networks_popup.handle_event(event, |network| {
                let network_str = self.form.get_text_mut(FormItem::Network);
                *network_str = network.name.clone();
                self.form.advance_cursor();
            })?);
        } else if self.tx_popup.is_open() {
            let r = self.tx_popup.handle_event(
                (event, area, tr, sd, ss),
                |_| {},
                |_| {},
                || {},
                || {},
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
                    .set_items(Some(NetworkStore::load_networks(ss.testnet_mode)?));
            } else {
                self.form.handle_event(event, |label, form| {
                    if label == FormItem::SendMessageButton {
                        let to = form.get_text(FormItem::To);
                        let message = form.get_text(FormItem::Message);
                        let network_name = form.get_text(FormItem::Network);
                        if message.is_empty() {
                            return Err("Message cannot be empty".into());
                        }

                        self.tx_popup.set_tx_req(
                            NetworkStore::get(network_name)?,
                            TransactionRequest::default().to(to.parse()?).input(
                                TransactionInput::from(Bytes::from(
                                    message.to_owned().into_bytes(),
                                )),
                            ),
                        );
                        self.tx_popup.open();
                    }

                    Ok(())
                })?;
            }
        }

        Ok(result)
    }

    fn render_component(
        &self,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
        shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf, &shared_state.theme);

        self.address_book_popup
            .render(area, buf, &shared_state.theme.popup_bg());

        self.networks_popup.render(area, buf, &shared_state.theme);

        self.tx_popup.render(area, buf, &shared_state.theme);

        area
    }
}
