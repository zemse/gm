use crate::app::SharedState;
use crate::pages::tx_popup::TxPopup;
use crate::widgets::{address_book_popup, networks_popup, AddressBookPopup, NetworksPopup};
use crate::{
    events::Event,
    traits::{Actions, Component},
};
use gm_ratatui_extra::act::Act;
use gm_ratatui_extra::form::{Form, FormWidget};
use gm_ratatui_extra::thematize::Thematize;
use gm_ratatui_extra::widgets::form::FormItemIndex;
use gm_utils::alloy::StringExt;
use gm_utils::disk_storage::DiskStorageInterface;
use gm_utils::network::{Network, NetworkStore};

use super::address_book::AddressBookMenuItem;
use crate::Result;

use alloy::primitives::Bytes;
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use ratatui::layout::Rect;
use std::sync::mpsc;
use std::sync::{atomic::AtomicBool, Arc};
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
        event: &Event,
        area: Rect,
        tr: &mpsc::Sender<Event>,
        sd: &Arc<AtomicBool>,
        ss: &SharedState,
    ) -> Result<Actions> {
        let mut result = Actions::default();

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
            result.merge(self.address_book_popup.handle_event(
                event.key_event(),
                |entry| -> crate::Result<()> {
                    let to_address = self.form.get_text_mut(FormItem::To);
                    *to_address = entry.address()?.to_string();
                    self.form.advance_cursor();
                    Ok(())
                },
            )?);
        } else if self.networks_popup.is_open() {
            result.merge(self.networks_popup.handle_event(
                event.key_event(),
                |network| -> crate::Result<()> {
                    let network_str = self.form.get_text_mut(FormItem::Network);
                    *network_str = network.name.clone();
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
                    event.key_event(),
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
                                Network::from_name(network_name)?,
                                TransactionRequest::default()
                                    .to(to.parse_as_address()?)
                                    .input(TransactionInput::from(Bytes::from(
                                        message.to_owned().into_bytes(),
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
        buf: &mut ratatui::prelude::Buffer,
        shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf, &shared_state.theme);

        self.address_book_popup
            .render(area, buf, &shared_state.theme.popup());

        self.networks_popup
            .render(area, buf, &shared_state.theme.popup());

        self.tx_popup.render(area, buf, &shared_state.theme.popup());

        area
    }
}
