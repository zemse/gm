use crate::app::SharedState;
use crate::pages::tx_popup::TxPopup;
use crate::traits::{Actions, Component};
use crate::widgets::{address_book_popup, assets_popup, AddressBookPopup, AssetsPopup};
use crate::{AppEvent, Result};
use alloy::primitives::utils::parse_units;
use alloy::primitives::{Bytes, U256};
use alloy::rpc::types::TransactionRequest;
use gm_ratatui_extra::act::Act;
use gm_ratatui_extra::form::{Form, FormItemIndex, FormWidget};
use gm_utils::alloy::StringExt;
use gm_utils::assets::{Asset, TokenAddress};
use gm_utils::erc20;
use gm_utils::network::Network;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::sync::mpsc;
use strum::{Display, EnumIter};
use tokio_util::sync::CancellationToken;

use super::address_book::AddressBookMenuItem;

#[derive(Debug, Display, EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    To,
    AssetType,
    Amount,
    ErrorText,
    TransferButton,
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
            FormItem::Heading => FormWidget::Heading("Transfer Assets"),
            FormItem::To => FormWidget::InputBox {
                label: "To",
                text: String::new(),
                empty_text: Some("<press SPACE to select from address book>"),
                currency: None,
            },
            FormItem::AssetType => FormWidget::DisplayBox {
                label: "Asset Type",
                text: String::new(),
                empty_text: Some("<press SPACE to select from your assets>"),
            },
            FormItem::Amount => FormWidget::InputBox {
                label: "Amount",
                text: String::new(),
                empty_text: None,
                currency: None,
            },
            FormItem::ErrorText => FormWidget::ErrorText(String::new()),
            FormItem::TransferButton => FormWidget::Button {
                label: "Transfer",
                hover_focus: false,
            },
        };
        Ok(widget)
    }
}

#[derive(Debug)]
pub struct AssetTransferPage {
    pub form: Form<FormItem, crate::Error>,
    pub asset: Option<Asset>, // TODO see if we can avoid this here
    pub address_book_popup: AddressBookPopup,
    pub asset_popup: AssetsPopup,
    pub tx_popup: TxPopup,
}

impl AssetTransferPage {
    fn try_default() -> crate::Result<Self> {
        Ok(Self {
            form: Form::init(|_| Ok(()))?,
            asset: None,
            address_book_popup: address_book_popup(),
            asset_popup: assets_popup(),
            tx_popup: TxPopup::default(),
        })
    }
}

impl AssetTransferPage {
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(asset: &Asset) -> crate::Result<Self> {
        let mut page = Self::try_default()?;
        page.asset = Some(asset.clone());

        // Update the form with the asset type, this is because the `asset` is
        // not directly linked to the ASSET_TYPE in form state
        *page.form.get_text_mut(FormItem::AssetType) = format!("{}", asset.r#type);
        *page
            .form
            .get_currency_mut(FormItem::Amount)
            .expect("currency not found in this input entry, please check idx") =
            Some(asset.r#type.symbol.clone());

        Ok(page)
    }
}

impl Component for AssetTransferPage {
    fn set_focus(&mut self, focus: bool) {
        self.form.set_form_focus(focus);
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: ratatui::prelude::Rect,
        tr: &mpsc::Sender<AppEvent>,
        sd: &CancellationToken,
        ss: &SharedState,
    ) -> Result<Actions> {
        let mut result = Actions::default();

        if self.address_book_popup.is_open() {
            result.merge(self.address_book_popup.handle_event(
                event.key_event(),
                |entry| -> crate::Result<()> {
                    *self.form.get_text_mut(FormItem::To) = entry.address()?.to_string();
                    self.form.advance_cursor();
                    Ok(())
                },
            )?);
        } else if self.asset_popup.is_open() {
            result.merge(self.asset_popup.handle_event(
                event.key_event(),
                |asset| -> crate::Result<()> {
                    self.asset = Some(asset.clone());
                    *self.form.get_text_mut(FormItem::AssetType) = format!("{}", asset.r#type);
                    *self
                        .form
                        .get_currency_mut(FormItem::Amount)
                        .expect("currency not found in this input entry, please check idx") =
                        Some(asset.r#type.symbol.clone());
                    self.form.advance_cursor();
                    Ok(())
                },
            )?);
        } else if self.tx_popup.is_open() {
            let is_confirmed = self.tx_popup.is_confirmed();
            let r = self.tx_popup.handle_event(
                (event, area, tr, sd, ss),
                |_| Ok(()),
                |_| Ok(()),
                |_, _, _| Ok(()),
                || Ok(()),
                || {
                    if is_confirmed {
                        result.page_pop = true;
                    }
                    Ok(())
                },
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
                result.ignore_esc();
            } else if self.form.is_focused(FormItem::AssetType) && event.is_space_or_enter_pressed()
            {
                self.asset_popup.open();
                self.asset_popup.set_items(ss.assets_read()?);
                result.ignore_esc();
            } else {
                let r = self.form.handle_event(
                    event.input_event(),
                    area,
                    |_, _| Ok(()),
                    |label, form| {
                        if label == FormItem::TransferButton {
                            let to = form.get_text(FormItem::To);
                            let asset =
                                self.asset.as_ref().ok_or(crate::Error::AssetNotSelected)?;
                            let amount = parse_units(
                                form.get_text(FormItem::Amount),
                                asset.r#type.decimals,
                            )?;

                            // TODO change erc20 logic here
                            let (to, calldata, value) = match asset.r#type.token_address {
                                TokenAddress::Native => {
                                    (to.parse_as_address()?, Bytes::new(), amount.get_absolute())
                                }
                                TokenAddress::Contract(address) => (
                                    address,
                                    erc20::encode_transfer(
                                        to.parse_as_address()?,
                                        amount.get_absolute(),
                                    ),
                                    U256::ZERO,
                                ),
                            };

                            if self.tx_popup.is_not_sent() || self.tx_popup.is_confirmed() {
                                self.tx_popup.set_tx_req(
                                    Network::from_name(&asset.r#type.network)?,
                                    TransactionRequest::default()
                                        .to(to)
                                        .value(value)
                                        .input(calldata.into()),
                                );
                            }

                            self.tx_popup.open();
                        }
                        Ok(())
                    },
                )?;
                result.merge(r);
            }
        }

        // Check for amount to be greateer than balance
        if let Some(asset) = &self.asset {
            let amount = self.form.get_text(FormItem::Amount);
            match parse_units(amount, asset.r#type.decimals) {
                Err(e) => {
                    *self.form.get_text_mut(FormItem::ErrorText) = format!("Invalid amount: {e}");
                }
                Ok(amount) => {
                    if amount.get_absolute() > asset.value {
                        *self.form.get_text_mut(FormItem::ErrorText) =
                            "Amount exceeds balance".to_string();
                    } else {
                        self.form.get_text_mut(FormItem::ErrorText).clear();
                    }
                }
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
            .render(popup_area, buf, &shared_state.theme);

        self.asset_popup
            .render(popup_area, buf, &shared_state.theme);

        self.tx_popup.render(popup_area, buf, &shared_state.theme);

        area
    }
}
