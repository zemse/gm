use crate::disk::DiskInterface;
use crate::network::{Network, NetworkStore, Token};
use crate::tui::app::pages::token::TokenPage;
use crate::tui::app::pages::Page;
use crate::tui::app::widgets::confirm_popup::ConfirmPopup;
use crate::tui::app::widgets::form::{Form, FormItemIndex, FormWidget};
use crate::tui::app::SharedState;
use crate::tui::traits::{Component, HandleResult};
use crate::tui::Event;
use alloy::primitives::Address;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use strum_macros::EnumIter;

#[derive(EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    Name,
    Symbol,
    Decimals,
    ContractAddress,
    SaveButton,
    RemoveButton,
    ErrorText, //TODO: Add tokens
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
            FormItem::Heading => FormWidget::Heading("Edit Network"),
            FormItem::Name => FormWidget::InputBox {
                label: "Name",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::Symbol => FormWidget::InputBox {
                label: "Symbol",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::Decimals => FormWidget::InputBox {
                label: "Decimals",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::ContractAddress => FormWidget::InputBox {
                label: "Contract Address",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::SaveButton => FormWidget::Button { label: "Save" },
            FormItem::RemoveButton => FormWidget::Button { label: "Remove" },
            FormItem::ErrorText => FormWidget::ErrorText(String::new()),
        };
        Ok(widget)
    }
}
pub struct TokenCreatePage {
    pub form: Form<FormItem>,
    pub token: Token,
    pub token_index: usize,
    pub network: Network,
    pub network_index: usize,
    pub remove_popup: ConfirmPopup,
}
impl TokenCreatePage {
    pub fn new(token_index: usize, network_index: usize, network: Network) -> crate::Result<Self> {
        let token = network.tokens.get(token_index).cloned().unwrap_or_default();
        Ok(Self {
            form: Form::init(|form| {
                *form.get_text_mut(FormItem::Name) = token.name.clone();
                *form.get_text_mut(FormItem::Symbol) = token.symbol.clone();
                *form.get_text_mut(FormItem::Decimals) = token.decimals.to_string();
                *form.get_text_mut(FormItem::ContractAddress) = token.contract_address.to_string();
                // if network.tokens.get(token_index).is_none() {
                //     form.hide_item(FormItem::RemoveButton);
                // }
                Ok(())
            })?,
            token_index,
            remove_popup: ConfirmPopup::new(
                "Remove Token",
                "Are you sure you want to remove this token?".to_string(),
                "Remove",
                "Cancel",
            ),
            token,
            network,
            network_index,
        })
    }
    pub fn token(form: &Form<FormItem>) -> Token {
        Token {
            name: form.get_text(FormItem::Name).clone(),
            symbol: form.get_text(FormItem::Symbol).clone(),
            decimals: form
                .get_text(FormItem::Decimals)
                .parse()
                .unwrap_or_default(),
            contract_address: form
                .get_text(FormItem::ContractAddress)
                .clone()
                .parse::<Address>()
                .unwrap_or_default(),
        }
    }
}
impl Component for TokenCreatePage {
    fn handle_event(
        &mut self,
        event: &Event,
        area: Rect,
        _transmitter: &Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        let mut handle_result = HandleResult::default();
        if self.remove_popup.is_open() {
            let r = self.remove_popup.handle_event(
                event,
                area,
                || {
                    if self.network.tokens.get(self.token_index).is_some() {
                        self.network.tokens.remove(self.token_index);
                        let mut config = NetworkStore::load()?;
                        config.networks[self.network_index] = self.network.clone();
                        config.save()?;
                    }
                    handle_result.page_pops = 1;
                    handle_result.page_inserts.push(Page::Token(TokenPage::new(
                        self.network_index,
                        self.network.clone(),
                    )?));
                    handle_result.reload = true;
                    Ok(())
                },
                || Ok(()),
            )?;
            handle_result.merge(r);
        }
        self.form.handle_event(
            event,
            |_, _| Ok(()),
            |label, form| {
                if label == FormItem::SaveButton {
                    let token = Self::token(form);
                    if self.network.tokens.get(self.token_index).is_some() {
                        self.network.tokens[self.token_index] = token;
                    } else {
                        self.network.tokens.push(token);
                    }

                    let mut config = NetworkStore::load()?;
                    config.networks[self.network_index] = self.network.clone();
                    config.save()?;

                    handle_result.page_pops = 1;
                    handle_result.page_inserts.push(Page::Token(TokenPage::new(
                        self.network_index,
                        self.network.clone(),
                    )?));
                    handle_result.reload = true;
                }
                if label == FormItem::RemoveButton {
                    self.remove_popup.open();
                }

                Ok(())
            },
        )?;
        Ok(handle_result)
    }
    fn render_component(&self, area: Rect, buf: &mut Buffer, s: &SharedState) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf, &s.theme);
        self.remove_popup.render(area, buf, &s.theme.popup());
        area
    }
}
