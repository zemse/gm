use crate::app::SharedState;
use crate::pages::token::TokenPage;
use crate::pages::Page;
use crate::post_handle_event::PostHandleEventActions;
use crate::traits::Component;
use crate::AppEvent;

use gm_ratatui_extra::boolean_input::BooleanInput;
use gm_ratatui_extra::button::Button;
use gm_ratatui_extra::confirm_popup::{ConfirmPopup, ConfirmResult};
use gm_ratatui_extra::form::{Form, FormEvent, FormItemIndex, FormWidget};
use gm_ratatui_extra::input_box::InputBox;
use gm_ratatui_extra::thematize::Thematize;
use gm_utils::disk_storage::DiskStorageInterface;
use gm_utils::network::{Network, NetworkStore, Token};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::borrow::Cow;
use std::ops::Not;
use std::sync::mpsc::Sender;
use strum::{Display, EnumIter};
use tokio_util::sync::CancellationToken;

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
    LineBreak1,
    SaveButton,
    LineBreak2,
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
                widget: InputBox::new("Name"),
            },
            FormItem::NameAlchemy => FormWidget::InputBox {
                widget: InputBox::new("Name Alchemy"),
            },
            FormItem::NameAliases => FormWidget::InputBox {
                widget: InputBox::new("Name Aliases"),
            },
            FormItem::ChainId => FormWidget::InputBox {
                widget: InputBox::new("Chain Id"),
            },
            FormItem::Symbol => FormWidget::InputBox {
                widget: InputBox::new("Symbol"),
            },
            FormItem::NativeDecimals => FormWidget::InputBox {
                widget: InputBox::new("Native Decimals"),
            },
            FormItem::PriceTicker => FormWidget::InputBox {
                widget: InputBox::new("Price Ticker"),
            },
            FormItem::RpcUrl => FormWidget::InputBox {
                widget: InputBox::new("RPC Url"),
            },
            FormItem::RpcAlchemy => FormWidget::InputBox {
                widget: InputBox::new("RPC Alchemy Url"),
            },
            FormItem::RpcInfura => FormWidget::InputBox {
                widget: InputBox::new("RPC Infura Url"),
            },
            FormItem::ExplorerUrl => FormWidget::InputBox {
                widget: InputBox::new("Explorer Url"),
            },
            FormItem::IsTestnet => FormWidget::BooleanInput {
                widget: BooleanInput::new("Is Testnet", false),
            },
            FormItem::RpcPort => FormWidget::InputBox {
                widget: InputBox::new("RPC Port"),
            },
            FormItem::ChainlinkNativePriceFeed => FormWidget::InputBox {
                widget: InputBox::new("Chainlink Native Price Feed"),
            },
            FormItem::ChainlinkNativePriceFeedDecimals => FormWidget::InputBox {
                widget: InputBox::new("Chainlink Native Price Feed Decimals"),
            },
            FormItem::TokensButton => FormWidget::Button {
                widget: Button::new("Tokens"),
            },
            FormItem::LineBreak1 => FormWidget::LineBreak,
            FormItem::SaveButton => FormWidget::Button {
                widget: Button::new("Save"),
            },
            FormItem::LineBreak2 => FormWidget::LineBreak,
            FormItem::RemoveButton => FormWidget::Button {
                widget: Button::new("Remove"),
            },
            FormItem::ErrorText => FormWidget::ErrorText(String::new()),
        };
        Ok(widget)
    }
}

#[derive(Debug)]
pub struct NetworkCreatePage {
    pub is_new: bool,
    pub form: Form<FormItem, crate::Error>,
    pub tokens: Vec<Token>,
    pub network_index: usize,
    pub network: Network,
    pub remove_popup: ConfirmPopup,
}
impl NetworkCreatePage {
    pub fn new(is_new: bool, network_index: usize, network: Network) -> crate::Result<Self> {
        let config = NetworkStore::load()?;

        Ok(Self {
            is_new,
            network: network.clone(),
            form: Form::init(|form| {
                form.set_text(FormItem::Name, network.name.clone());
                if let Some(name_alchemy) = network.name_alchemy {
                    form.set_text(FormItem::NameAlchemy, name_alchemy);
                }
                if network.name_aliases.is_empty().not() {
                    form.set_text(FormItem::NameAliases, network.name_aliases[0].clone());
                }

                form.set_text(FormItem::ChainId, network.chain_id.to_string());
                if let Some(symbol) = network.symbol {
                    form.set_text(FormItem::Symbol, symbol);
                }
                if let Some(native_decimals) = network.native_decimals {
                    form.set_text(FormItem::NativeDecimals, native_decimals.to_string());
                }
                if let Some(price_ticker) = network.price_ticker {
                    form.set_text(FormItem::PriceTicker, price_ticker);
                }
                if let Some(rpc_url) = network.rpc_url {
                    form.set_text(FormItem::RpcUrl, rpc_url);
                }
                if let Some(rpc_alchemy) = network.rpc_alchemy {
                    form.set_text(FormItem::RpcAlchemy, rpc_alchemy);
                }
                if let Some(rpc_infura) = network.rpc_infura {
                    form.set_text(FormItem::RpcInfura, rpc_infura);
                }
                if let Some(explorer_url) = network.explorer_url {
                    form.set_text(FormItem::ExplorerUrl, explorer_url);
                }
                if let Some(rpc_port) = network.rpc_port {
                    form.set_text(FormItem::RpcPort, rpc_port.to_string());
                }
                if let Some(chainlink_native_price_feed) = network.chainlink_native_price_feed {
                    form.set_text(
                        FormItem::ChainlinkNativePriceFeed,
                        chainlink_native_price_feed.to_string(),
                    )
                }
                if let Some(chainlink_native_price_feed_decimals) =
                    network.chainlink_native_price_feed_decimals
                {
                    form.set_text(
                        FormItem::ChainlinkNativePriceFeedDecimals,
                        chainlink_native_price_feed_decimals.to_string(),
                    )
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
                true,
            ),
        })
    }

    fn network(form: &Form<FormItem, crate::Error>, tokens: &[Token]) -> crate::Result<Network> {
        Ok(Network {
            name: form.get_text(FormItem::Name).to_string(),
            name_alchemy: form
                .get_text(FormItem::NameAlchemy)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::NameAlchemy).to_string()),
            name_aliases: vec![form.get_text(FormItem::NameAliases).to_string()],
            chain_id: form.get_text(FormItem::ChainId).parse().unwrap_or_default(),
            symbol: form
                .get_text(FormItem::Symbol)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::Symbol).to_string()),
            native_decimals: form.get_text(FormItem::NativeDecimals).parse().ok(),
            price_ticker: form
                .get_text(FormItem::PriceTicker)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::PriceTicker).to_string()),
            rpc_url: form
                .get_text(FormItem::RpcUrl)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::RpcUrl).to_string()),
            rpc_alchemy: form
                .get_text(FormItem::RpcAlchemy)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::RpcAlchemy).to_string()),
            rpc_infura: form
                .get_text(FormItem::RpcInfura)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::RpcInfura).to_string()),
            explorer_url: form
                .get_text(FormItem::ExplorerUrl)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::ExplorerUrl).to_string()),
            is_testnet: form.get_boolean(FormItem::IsTestnet),
            rpc_port: form
                .get_text(FormItem::RpcPort)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::RpcPort).parse())
                .transpose()?,
            chainlink_native_price_feed: form
                .get_text(FormItem::ChainlinkNativePriceFeed)
                .is_empty()
                .not()
                .then(|| form.get_text(FormItem::ChainlinkNativePriceFeed).parse())
                .transpose()
                .map_err(crate::Error::FromHexError)?,
            chainlink_native_price_feed_decimals: form
                .get_text(FormItem::ChainlinkNativePriceFeedDecimals)
                .is_empty()
                .not()
                .then(|| {
                    form.get_text(FormItem::ChainlinkNativePriceFeedDecimals)
                        .parse()
                })
                .transpose()?,
            tokens: tokens.to_owned(),
        })
    }
}

impl Component for NetworkCreatePage {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Create")
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        popup_area: Rect,
        _transmitter: &Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let mut actions = PostHandleEventActions::default();
        if self.remove_popup.is_open() {
            if let Some(ConfirmResult::Confirmed) =
                self.remove_popup
                    .handle_event(event.input_event(), popup_area, &mut actions)?
            {
                let mut config = NetworkStore::load()?;
                if config.networks.get(self.network_index).is_some() {
                    config.networks.remove(self.network_index);
                }
                let _ = config.save();
                actions.page_pop();
                actions.reload();
            }
        }
        if let Some(FormEvent::ButtonPressed(label)) = self.form.handle_event(
            event.widget_event().as_ref(),
            area,
            popup_area,
            &mut actions,
        )? {
            match label {
                FormItem::SaveButton => {
                    if self.form.get_text(FormItem::Name).is_empty() {
                        self.form.set_text(
                            FormItem::ErrorText,
                            "Please enter name, you cannot leave it empty".to_string(),
                        );
                    } else if self.form.get_text(FormItem::ChainId).is_empty() {
                        self.form.set_text(
                            FormItem::ErrorText,
                            "Please enter chain id, you cannot leave it empty".to_string(),
                        );
                    } else {
                        let mut config = NetworkStore::load()?;
                        if config.networks.get(self.network_index).is_some() {
                            config.networks[self.network_index] =
                                Self::network(&self.form, &self.tokens)?;
                        } else {
                            config
                                .networks
                                .push(Self::network(&self.form, &self.tokens)?);
                        }
                        let _ = config.save();
                        actions.page_pop();
                        actions.reload();
                    }
                }
                FormItem::TokensButton => {
                    let network = Self::network(&self.form, &self.tokens)?;
                    actions.page_insert(Page::Token(TokenPage::new(self.network_index, network)?));
                    // TODO should not allow going to tokens if network is not saved, do something about this
                }
                FormItem::RemoveButton => {
                    self.remove_popup.open();
                }
                _ => {}
            }
        }

        Ok(actions)
    }
    fn render_component(
        &self,
        area: Rect,
        popup_area: Rect,
        buf: &mut Buffer,
        s: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, popup_area, buf, &s.theme);
        self.remove_popup.render(popup_area, buf, &s.theme.popup());
        area
    }
}
