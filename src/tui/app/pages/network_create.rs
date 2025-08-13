use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::ops::Not;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::disk::DiskInterface;
use crate::network::{Network, NetworkStore, Token};
use crate::tui::app::pages::token::TokenPage;
use crate::tui::app::pages::Page;
use crate::tui::app::widgets::confirm_popup::ConfirmPopup;
use crate::tui::app::widgets::form::{Form, FormItemIndex, FormWidget};
use crate::tui::app::SharedState;
use crate::tui::traits::{Component, HandleResult};
use crate::tui::Event;
use strum::EnumIter;

#[derive(EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    Name,
    NameAlchemy,
    NameAliases,
    ChainId,
    Symbol,
    NativeDecimals,
    PriceTicker,
    RpcUrl,
    RpcAlchemy,
    RpcInfura,
    ExplorerUrl,
    IsTestnet,
    TokensButton,
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
            FormItem::NameAlchemy => FormWidget::InputBox {
                label: "Name Alchemy",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::NameAliases => FormWidget::InputBox {
                label: "Name Aliases",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::ChainId => FormWidget::InputBox {
                label: "Chain Id",
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
            FormItem::NativeDecimals => FormWidget::InputBox {
                label: "Native Decimals",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::PriceTicker => FormWidget::InputBox {
                label: "Price Ticker",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::RpcUrl => FormWidget::InputBox {
                label: "RPC Url",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::RpcAlchemy => FormWidget::InputBox {
                label: "RPC Alchemy Url",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::RpcInfura => FormWidget::InputBox {
                label: "RPC Infura Url",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::ExplorerUrl => FormWidget::InputBox {
                label: "Explorer Url",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::IsTestnet => FormWidget::BooleanInput {
                label: "Testnet",
                value: false,
            },
            FormItem::TokensButton => FormWidget::Button { label: "Tokens" },
            FormItem::SaveButton => FormWidget::Button { label: "Save" },
            FormItem::RemoveButton => FormWidget::Button { label: "Remove" },
            FormItem::ErrorText => FormWidget::ErrorText(String::new()),
        };
        Ok(widget)
    }
}
pub struct NetworkCreatePage {
    pub form: Form<FormItem>,
    pub tokens: Vec<Token>,
    pub network_index: usize,
    pub network: Network,
    pub remove_popup: ConfirmPopup,
}
impl NetworkCreatePage {
    pub fn new(network_index: usize, network: Network) -> crate::Result<Self> {
        let config = NetworkStore::load()?;

        Ok(Self {
            network: network.clone(),
            form: Form::init(|form| {
                *form.get_text_mut(FormItem::Name) = network.name.clone();
                if let Some(name_alchemy) = network.name_alchemy {
                    *form.get_text_mut(FormItem::NameAlchemy) = name_alchemy;
                }
                if network.name_aliases.is_empty().not() {
                    *form.get_text_mut(FormItem::NameAliases) = network.name_aliases[0].clone();
                }

                *form.get_text_mut(FormItem::ChainId) = network.chain_id.to_string();
                if let Some(symbol) = network.symbol {
                    *form.get_text_mut(FormItem::Symbol) = symbol;
                }
                if let Some(native_decimals) = network.native_decimals {
                    *form.get_text_mut(FormItem::NativeDecimals) = native_decimals.to_string();
                }
                if let Some(price_ticker) = network.price_ticker {
                    *form.get_text_mut(FormItem::PriceTicker) = price_ticker;
                }
                if let Some(rpc_url) = network.rpc_url {
                    *form.get_text_mut(FormItem::RpcUrl) = rpc_url;
                }
                if let Some(rpc_alchemy) = network.rpc_alchemy {
                    *form.get_text_mut(FormItem::RpcAlchemy) = rpc_alchemy;
                }
                if let Some(rpc_infura) = network.rpc_infura {
                    *form.get_text_mut(FormItem::RpcInfura) = rpc_infura;
                }
                if let Some(explorer_url) = network.explorer_url {
                    *form.get_text_mut(FormItem::ExplorerUrl) = explorer_url;
                }
                if config.networks.get(network_index).is_none() {
                    form.hide_item(FormItem::RemoveButton);
                }
                *form.get_boolean_mut(FormItem::IsTestnet) = network.is_testnet;
                Ok(())
            })?,
            network_index,
            tokens: network.tokens,
            remove_popup: ConfirmPopup::new(
                "Remove Network",
                format!(
                    "Are you sure you want to remove the network '{}'?",
                    network.name
                ),
                "Remove",
                "Cancel",
            ),
        })
    }
    fn network(form: &Form<FormItem>, tokens: &[Token]) -> Network {
        Network {
            name: form.get_text(FormItem::Name).clone(),
            name_alchemy: form
                .get_text(FormItem::NameAlchemy)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::NameAlchemy).clone()),
            name_aliases: vec![form.get_text(FormItem::NameAliases).clone()],
            chain_id: form.get_text(FormItem::ChainId).parse().unwrap_or_default(),
            symbol: form
                .get_text(FormItem::Symbol)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::Symbol).clone()),
            native_decimals: form.get_text(FormItem::NativeDecimals).parse().ok(),
            price_ticker: form
                .get_text(FormItem::PriceTicker)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::PriceTicker).clone()),
            rpc_url: form
                .get_text(FormItem::RpcUrl)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::RpcUrl).clone()),
            rpc_alchemy: form
                .get_text(FormItem::RpcAlchemy)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::RpcAlchemy).clone()),
            rpc_infura: form
                .get_text(FormItem::RpcInfura)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::RpcInfura).clone()),
            explorer_url: form
                .get_text(FormItem::ExplorerUrl)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::ExplorerUrl).clone()),
            is_testnet: form.get_boolean(FormItem::IsTestnet),
            tokens: tokens.to_owned(),
        }
    }
}

impl Component for NetworkCreatePage {
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
            self.remove_popup.handle_event(
                event,
                area,
                || {
                    let mut config = NetworkStore::load()?;
                    if config.networks.get(self.network_index).is_some() {
                        config.networks.remove(self.network_index);
                    }
                    let _ = config.save();
                    handle_result.page_pops = 1;
                    handle_result.reload = true;
                    Ok(())
                },
                || Ok(()),
            )?;
        }
        self.form.handle_event(event, |label, form| {
            match label {
                FormItem::SaveButton => {
                    if form.get_text(FormItem::Name).is_empty() {
                        let error = form.get_text_mut(FormItem::ErrorText);
                        *error = "Please enter name, you cannot leave it empty".to_string();
                    } else if form.get_text(FormItem::ChainId).is_empty() {
                        let error = form.get_text_mut(FormItem::ErrorText);
                        *error = "Please enter chain id, you cannot leave it empty".to_string();
                    } else {
                        let mut config = NetworkStore::load()?;
                        if config.networks.get(self.network_index).is_some() {
                            config.networks[self.network_index] = Self::network(form, &self.tokens);
                        } else {
                            config.networks.push(Self::network(form, &self.tokens));
                        }
                        let _ = config.save();
                        handle_result.page_pops = 1;
                        handle_result.reload = true;
                    }
                }
                FormItem::TokensButton => {
                    let network = Self::network(form, &self.tokens);
                    handle_result.page_pops = 1;
                    handle_result
                        .page_inserts
                        .push(Page::Token(TokenPage::new(self.network_index, network)?));
                    handle_result.reload = true;
                }
                FormItem::RemoveButton => {
                    self.remove_popup.open();
                }
                _ => {}
            }

            Ok(())
        })?;

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
