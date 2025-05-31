use crate::disk::DiskInterface;
use crate::network::{Network, NetworkStore};

use crate::tui::app::widgets::filter_select_popup::FilterSelectPopup;
use crate::tui::app::widgets::form::FormItemIndex;

use crate::tui::app::SharedState;
use crate::tui::{
    app::widgets::form::{Form, FormWidget}, // <- Using your custom form system
    events::Event,
    traits::{Component, HandleResult},
};

use crate::Result;

use alloy::primitives::{Bytes, TxKind, U256};

use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use std::sync::mpsc;
use std::sync::{atomic::AtomicBool, Arc};
use strum::EnumIter;

use super::address_book::AddressBookMenuItem;
use super::transaction::TransactionPage;
use super::Page;

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
impl From<FormItem> for FormWidget {
    fn from(value: FormItem) -> Self {
        match value {
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
        }
    }
}

pub struct SendMessagePage {
    pub form: Form<FormItem>,
    pub address_book_popup: FilterSelectPopup<AddressBookMenuItem>,
    pub networks_popup: FilterSelectPopup<Network>,
}

impl Default for SendMessagePage {
    fn default() -> Self {
        Self {
            form: Form::init(|_| {}),
            address_book_popup: FilterSelectPopup::new("Address Book"),
            networks_popup: FilterSelectPopup::new("Networks"),
        }
    }
}

impl Component for SendMessagePage {
    fn set_focus(&mut self, focus: bool) {
        self.form.set_form_focus(focus);
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        _tr: &mpsc::Sender<Event>,
        _sd: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> Result<HandleResult> {
        let mut result = HandleResult::default();

        #[allow(clippy::single_match)]
        match event {
            Event::ConfigUpdate => {
                let network_name = self.form.get_text(FormItem::Network);
                let network_store = NetworkStore::load();
                let network = network_store
                    .get_by_name(network_name)
                    .ok_or(crate::Error::NetworkNotFound(network_name.to_string()))?;

                if network.is_testnet != shared_state.testnet_mode {
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
        } else {
            // Handle form events
            if self.form.is_focused(FormItem::To)
                && self.form.get_text(FormItem::To).is_empty()
                && event.is_space_or_enter_pressed()
            {
                self.address_book_popup
                    .open(Some(AddressBookMenuItem::get_menu(
                        false,
                        shared_state.recent_addresses.clone(),
                    )));
            } else if self.form.is_focused(FormItem::Network) && event.is_space_or_enter_pressed() {
                self.networks_popup
                    .open(Some(NetworkStore::load_networks(shared_state.testnet_mode)));
            } else {
                self.form.handle_event(event, |label, form| {
                    if label == FormItem::SendMessageButton {
                        let to = form.get_text(FormItem::To);
                        let message = form.get_text(FormItem::Message);
                        let network_name = form.get_text(FormItem::Network);
                        if message.is_empty() {
                            return Err("Message cannot be empty".into());
                        }
                        result
                            .page_inserts
                            .push(Page::Transaction(TransactionPage::new(
                                network_name,
                                TxKind::Call(to.parse()?),
                                Bytes::from(message.to_owned().into_bytes()),
                                U256::ZERO,
                            )?));
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
        _shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf);

        self.address_book_popup.render(area, buf);

        self.networks_popup.render(area, buf);

        area
    }
}
