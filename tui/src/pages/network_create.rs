use gm_ratatui_extra::act::Act;
use gm_ratatui_extra::confirm_popup::ConfirmPopup;
use gm_ratatui_extra::form::{Form, FormItemIndex, FormWidget};
use gm_ratatui_extra::thematize::Thematize;
use gm_utils::disk_storage::DiskStorageInterface;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::ops::Not;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::app::SharedState;
use crate::pages::token::TokenPage;
use crate::pages::Page;
use crate::traits::{Actions, Component};
use crate::Event;
use gm_utils::network::{Network, NetworkStore, Token};
use strum::{Display, EnumIter};

#[derive(Debug, Display, EnumIter, PartialEq)]
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
    RpcPort,
    ChainlinkNativePriceFeed,
    ChainlinkNativePriceFeedDecimals,
    TokensButton,
    SaveButton,
    RemoveButton,
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
            FormItem::RpcPort => FormWidget::InputBox {
                label: "RPC Port",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::ChainlinkNativePriceFeed => FormWidget::InputBox {
                label: "Chainlink Native Price Feed",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::ChainlinkNativePriceFeedDecimals => FormWidget::InputBox {
                label: "Chainlink Native Price Feed Decimals",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::TokensButton => FormWidget::Button { label: "Tokens" },
            FormItem::SaveButton => FormWidget::Button { label: "Save" },
            FormItem::RemoveButton => FormWidget::Button { label: "Remove" },
            FormItem::ErrorText => FormWidget::ErrorText(String::new()),
        };
        Ok(widget)
    }
}

#[derive(Debug)]
pub struct NetworkCreatePage {
    pub form: Form<FormItem, crate::Error>,
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
                if let Some(rpc_port) = network.rpc_port {
                    *form.get_text_mut(FormItem::RpcPort) = rpc_port.to_string();
                }
                if let Some(chainlink_native_price_feed) = network.chainlink_native_price_feed {
                    *form.get_text_mut(FormItem::ChainlinkNativePriceFeed) =
                        chainlink_native_price_feed.to_string()
                }
                if let Some(chainlink_native_price_feed_decimals) =
                    network.chainlink_native_price_feed_decimals
                {
                    *form.get_text_mut(FormItem::ChainlinkNativePriceFeedDecimals) =
                        chainlink_native_price_feed_decimals.to_string()
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

    fn network(form: &Form<FormItem, crate::Error>, tokens: &[Token]) -> crate::Result<Network> {
        Ok(Network {
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
            rpc_port: form
                .get_text(FormItem::RpcPort)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::RpcPort).clone().parse())
                .transpose()?,
            chainlink_native_price_feed: form
                .get_text(FormItem::ChainlinkNativePriceFeed)
                .is_empty()
                .not()
                .then(|| {
                    form.get_text(FormItem::ChainlinkNativePriceFeed)
                        .clone()
                        .parse()
                })
                .transpose()
                .map_err(crate::Error::FromHexError)?,
            chainlink_native_price_feed_decimals: form
                .get_text(FormItem::ChainlinkNativePriceFeedDecimals)
                .is_empty()
                .not()
                .then(|| {
                    form.get_text(FormItem::ChainlinkNativePriceFeedDecimals)
                        .clone()
                        .parse()
                })
                .transpose()?,
            tokens: tokens.to_owned(),
        })
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
    ) -> crate::Result<Actions> {
        let mut handle_result = Actions::default();
        if self.remove_popup.is_open() {
            let r = self.remove_popup.handle_event(
                event.key_event(),
                area,
                || -> crate::Result<()> {
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
            handle_result.merge(r);
        }
        let r = self.form.handle_event(
            event.key_event(),
            |_, _| Ok(()),
            |label, form| {
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
                                config.networks[self.network_index] =
                                    Self::network(form, &self.tokens)?;
                            } else {
                                config.networks.push(Self::network(form, &self.tokens)?);
                            }
                            let _ = config.save();
                            handle_result.page_pops = 1;
                            handle_result.reload = true;
                        }
                    }
                    FormItem::TokensButton => {
                        let network = Self::network(form, &self.tokens)?;
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
            },
        )?;
        handle_result.merge(r);

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
