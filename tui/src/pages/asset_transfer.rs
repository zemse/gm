use crate::app::SharedState;
use crate::pages::sign_tx_popup::{SignTxEvent, SignTxPopup};
use crate::post_handle_event::PostHandleEventActions;
use crate::traits::Component;
use crate::widgets::{address_book_popup, assets_popup, AddressBookPopup, AssetsPopup};
use crate::{AppEvent, Result};
use alloy::primitives::utils::parse_units;
use alloy::primitives::{Bytes, U256};
use alloy::rpc::types::TransactionRequest;
use gm_ratatui_extra::act::Act;
use gm_ratatui_extra::button::Button;
use gm_ratatui_extra::extensions::ThemedWidget;
use gm_ratatui_extra::form::{Form, FormEvent, FormItemIndex, FormWidget};
use gm_ratatui_extra::input_box::InputBox;
use gm_ratatui_extra::popup::PopupWidget;
use gm_utils::alloy::StringExt;
use gm_utils::assets::{Asset, TokenAddress};
use gm_utils::erc20;
use gm_utils::network::Network;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::borrow::Cow;
use std::sync::mpsc;
use strum::{Display, EnumIter};
use tokio_util::sync::CancellationToken;

use super::address_book::AddressBookMenuItem;

#[derive(Debug, Display, EnumIter, PartialEq)]
pub enum FormItem {
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
            FormItem::To => FormWidget::InputBox {
                widget: InputBox::new("To")
                    .with_empty_text("press SPACE to select from address book"),
            },
            FormItem::AssetType => FormWidget::DisplayBox {
                widget: InputBox::new("Asset Type")
                    .with_empty_text("press SPACE to select from your assets"),
            },
            FormItem::Amount => FormWidget::InputBox {
                widget: InputBox::new("Amount"),
            },
            FormItem::ErrorText => FormWidget::ErrorText(String::new()),
            FormItem::TransferButton => FormWidget::Button {
                widget: Button::new("Transfer"),
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
    pub tx_popup: SignTxPopup,
}

impl AssetTransferPage {
    fn try_default() -> crate::Result<Self> {
        Ok(Self {
            form: Form::init(|_| Ok(()))?,
            asset: None,
            address_book_popup: address_book_popup(),
            asset_popup: assets_popup(),
            tx_popup: SignTxPopup::Closed,
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
        page.form
            .set_text(FormItem::AssetType, format!("{}", asset.r#type));
        *page
            .form
            .get_currency_mut(FormItem::Amount)
            .expect("currency not found in this input entry, please check idx") =
            Some(asset.r#type.symbol.clone());

        Ok(page)
    }
}

impl Component for AssetTransferPage {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Transfer")
    }

    fn set_focus(&mut self, focus: bool) {
        self.form.set_form_focus(focus);
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        popup_area: Rect,
        _tr: &mpsc::Sender<AppEvent>,
        _sd: &CancellationToken,
        ss: &SharedState,
    ) -> Result<PostHandleEventActions> {
        let mut actions = PostHandleEventActions::default();

        if self.address_book_popup.is_open() {
            if let Some(selection) = self.address_book_popup.handle_event(
                event.input_event(),
                popup_area,
                &mut actions,
            )? {
                self.form
                    .set_text(FormItem::To, selection.address()?.to_string());
                self.form.advance_cursor();
            }
        } else if self.asset_popup.is_open() {
            if let Some(selection) =
                self.asset_popup
                    .handle_event(event.input_event(), popup_area, &mut actions)?
            {
                self.asset = Some(selection.as_ref().clone());

                self.form
                    .set_text(FormItem::AssetType, format!("{}", selection.r#type));
                *self
                    .form
                    .get_currency_mut(FormItem::Amount)
                    .expect("currency not found in this input entry, please check idx") =
                    Some(selection.r#type.symbol.clone());

                self.form.advance_cursor();
            }
        } else if self.tx_popup.is_open() {
            match self
                .tx_popup
                .handle_event(event, popup_area, &mut actions)?
            {
                Some(SignTxEvent::Confirmed(_)) => {
                    actions.refresh_assets();
                }
                Some(SignTxEvent::Cancelled) => {
                    actions.ignore_esc();
                }
                Some(SignTxEvent::Done) => {
                    actions.page_pop();
                    actions.ignore_esc();
                }
                _ => {}
            }
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
                actions.ignore_esc();
            } else if self.form.is_focused(FormItem::AssetType) && event.is_space_or_enter_pressed()
            {
                self.asset_popup.open();
                self.asset_popup.set_items(ss.assets_read()?);
                actions.ignore_esc();
            } else {
                // Handle form events
                if let Some(FormEvent::ButtonPressed(label)) = self.form.handle_event(
                    event.widget_event().as_ref(),
                    area,
                    popup_area,
                    &mut actions,
                )? {
                    if label == FormItem::TransferButton {
                        let to = self.form.get_text(FormItem::To);
                        let asset = self.asset.as_ref().ok_or(crate::Error::AssetNotSelected)?;
                        let amount = parse_units(
                            &self.form.get_text(FormItem::Amount),
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

                        // if self.tx_popup.is_not_sent() || self.tx_popup.is_confirmed() {
                        self.tx_popup = SignTxPopup::new(
                            ss.config.get_current_account()?,
                            Network::from_name(&asset.r#type.network)?,
                            TransactionRequest::default()
                                .to(to)
                                .value(value)
                                .input(calldata.into()),
                        );
                    }
                }
            }
        }

        // Check for amount to be greateer than balance
        if let Some(asset) = &self.asset {
            let amount = self.form.get_text(FormItem::Amount);
            match parse_units(&amount, asset.r#type.decimals) {
                Err(e) => {
                    self.form
                        .set_text(FormItem::ErrorText, format!("Invalid amount: {e}"));
                }
                Ok(amount) => {
                    if amount.get_absolute() > asset.value {
                        self.form
                            .set_text(FormItem::ErrorText, "Amount exceeds balance".to_string());
                    } else {
                        self.form.set_text(FormItem::ErrorText, "".to_string());
                    }
                }
            }
        }

        Ok(actions)
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
