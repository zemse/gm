use crate::tui::app::pages::transaction::TransactionPage;
use crate::tui::app::pages::Page;
use crate::tui::app::widgets::address_book_popup::AddressBookPopup;
use crate::tui::app::widgets::assets_popup::AssetsPopup;
use crate::tui::app::widgets::form::FormItemIndex;
use crate::tui::app::SharedState;
use crate::tui::{
    app::widgets::form::{Form, FormWidget},
    events::Event,
    traits::{Component, HandleResult},
};
use crate::utils::assets::{Asset, TokenAddress};
use crate::Result;
use alloy::primitives::utils::parse_units;
use alloy::primitives::{Bytes, TxKind, U256};
use alloy::sol;
use alloy::sol_types::SolCall;
use ratatui::widgets::Widget;
use std::sync::mpsc;
use std::sync::{atomic::AtomicBool, Arc};
use strum::EnumIter;

use super::address_book::AddressBookMenuItem;

#[derive(EnumIter, PartialEq)]
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
impl From<FormItem> for FormWidget {
    fn from(value: FormItem) -> Self {
        match value {
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
            FormItem::TransferButton => FormWidget::Button { label: "Transfer" },
        }
    }
}

pub struct AssetTransferPage {
    pub form: Form<FormItem>,
    pub asset: Option<Asset>, // TODO see if we can avoid this here
    pub address_book_popup: AddressBookPopup,
    pub asset_popup: AssetsPopup,
}

impl Default for AssetTransferPage {
    fn default() -> Self {
        Self {
            form: Form::init(|_| {}),
            asset: None,
            address_book_popup: AddressBookPopup::default(),
            asset_popup: AssetsPopup::default(),
        }
    }
}

impl AssetTransferPage {
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(asset: &Asset) -> Self {
        let mut page = Self::default();
        page.asset = Some(asset.clone());

        // Update the form with the asset type, this is because the `asset` is
        // not directly linked to the ASSET_TYPE in form state
        *page.form.get_text_mut(FormItem::AssetType) = format!("{}", asset.r#type);
        *page
            .form
            .get_currency_mut(FormItem::Amount)
            .expect("currency not found in this input entry, please check idx") =
            Some(asset.r#type.symbol.clone());

        page
    }
}

impl Component for AssetTransferPage {
    fn set_focus(&mut self, focus: bool) {
        self.form.set_form_focus(focus);
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: ratatui::prelude::Rect,
        _tr: &mpsc::Sender<Event>,
        _sd: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> Result<HandleResult> {
        let mut result = HandleResult::default();

        if self.address_book_popup.is_open() {
            result.merge(self.address_book_popup.handle_event(event, |entry| {
                *self.form.get_text_mut(FormItem::To) = entry.address_unwrap().to_string();
                self.form.advance_cursor();
            })?);
        } else if self.asset_popup.is_open() {
            result.merge(self.asset_popup.handle_event(event, |asset| {
                self.asset = Some(asset.clone());
                *self.form.get_text_mut(FormItem::AssetType) = format!("{}", asset.r#type);
                *self
                    .form
                    .get_currency_mut(FormItem::Amount)
                    .expect("currency not found in this input entry, please check idx") =
                    Some(asset.r#type.symbol.clone());
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
                result.esc_ignores = 1;
            } else if self.form.is_focused(FormItem::AssetType) && event.is_space_or_enter_pressed()
            {
                self.asset_popup.open(shared_state.assets.clone());
                result.esc_ignores = 1;
            } else {
                self.form.handle_event(event, |label, form| {
                if label == FormItem::TransferButton {
                    let to = form.get_text(FormItem::To);
                    let asset = self
                        .asset
                        .as_ref()
                        .ok_or(crate::Error::InternalErrorStr("No asset selected"))?;
                    let amount =
                        parse_units(form.get_text(FormItem::Amount), asset.r#type.decimals)?;

                    sol! {
                        interface IERC20 {
                            function balanceOf(address owner) external view returns (uint256);
                            function transfer(address to, uint256 amount) external returns (bool);
                        }
                    }

                    let (to, calldata, value) = match asset.r#type.token_address {
                        TokenAddress::Native => (
                            TxKind::Call(to.parse()?),
                            Bytes::new(),
                            amount.get_absolute(),
                        ),
                        TokenAddress::Contract(address) => {
                            let transfer_call = IERC20::transferCall {
                                to: to.parse()?,
                                amount: amount.get_absolute(),
                            };

                            let calldata = Bytes::from(transfer_call.abi_encode());

                            (TxKind::Call(address), calldata, U256::ZERO)
                        }
                    };

                    result
                        .page_inserts
                        .push(Page::Transaction(TransactionPage::new(
                            &asset.r#type.network,
                            to,
                            calldata,
                            value,
                        )?))
                }
                Ok(())
            })?;
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
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        _shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf);

        self.address_book_popup.render(area, buf);

        self.asset_popup.render(area, buf);

        area
    }
}
