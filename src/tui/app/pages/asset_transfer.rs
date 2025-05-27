use crate::disk::{AddressBook, AddressBookEntry, DiskInterface};
use crate::tui::app::pages::transaction::TransactionPage;
use crate::tui::app::pages::Page;
use crate::tui::app::widgets::filter_select::FilterSelect;
use crate::tui::app::widgets::form::FormItemIndex;
use crate::tui::app::widgets::popup::Popup;
use crate::tui::app::{Focus, SharedState};
use crate::tui::{
    app::widgets::form::{Form, FormWidget},
    events::Event,
    traits::{Component, HandleResult},
};
use crate::utils::assets::{Asset, TokenAddress};
use crate::utils::cursor::Cursor;
use crate::Result;
use alloy::primitives::utils::parse_units;
use alloy::primitives::{Bytes, TxKind, U256};
use alloy::sol;
use alloy::sol_types::SolCall;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::style::Color;
use ratatui::widgets::{Block, Widget};
use std::sync::mpsc;
use std::sync::{atomic::AtomicBool, Arc};
use strum::EnumIter;

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
    pub asset: Option<Asset>,
    /// Asset popup - we get asset details from `shared_state`
    pub show_asset_popup: bool,
    /// Address book popup state
    pub address_book: Option<AddressBook>,
    pub search_string: String,
    /// Reused for both Address book popup and Asset popup
    pub cursor: Cursor,
}

impl Default for AssetTransferPage {
    fn default() -> Self {
        Self {
            form: Form::init(),
            asset: None,
            address_book: None,
            cursor: Cursor::default(),
            search_string: String::new(),
            show_asset_popup: false,
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
    fn handle_event(
        &mut self,
        event: &Event,
        _tr: &mpsc::Sender<Event>,
        _sd: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> Result<HandleResult> {
        let mut result = HandleResult::default();

        if self.address_book.is_none() && !self.show_asset_popup {
            // Activate the address book popup if the user presses SPACE in the "To" field
            if self.form.is_focused(FormItem::To)
                && self.form.get_text(FormItem::To).is_empty()
                && (event.is_char_pressed(Some(' ')) || event.is_key_pressed(KeyCode::Enter))
            {
                self.address_book = Some(AddressBook::load());
                self.cursor = Cursor::default();
            }

            if self.form.is_focused(FormItem::AssetType)
                && (event.is_char_pressed(Some(' ')) || event.is_key_pressed(KeyCode::Enter))
            {
                self.show_asset_popup = true;
                self.cursor = Cursor::default();
            }

            // Keyboard events focus on the form
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
        } else if let Some(address_book) = self.address_book.as_ref() {
            // Keyboard events go to the address book popup
            // TODO refactor this code into FilterSelect module
            let list: Vec<&AddressBookEntry> = address_book
                .list()
                .iter()
                .filter(|entry| format!("{entry}").contains(self.search_string.as_str()))
                .collect();

            let cursor_max = list.len();
            self.cursor.handle(event, cursor_max);

            if let Event::Input(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char(char) => {
                            self.search_string.push(char);
                        }
                        KeyCode::Backspace => {
                            self.search_string.pop();
                        }
                        KeyCode::Enter => {
                            let to_address = self.form.get_text_mut(FormItem::To);
                            *to_address = list[self.cursor.current].address.to_string();
                            self.address_book = None;
                            self.form.advance_cursor();
                        }
                        _ => {}
                    }
                }
            }
        } else if self.show_asset_popup {
            // Keyboard events go to the asset popup
            if let Some(assets) = shared_state.assets.as_ref() {
                let cursor_max = assets.len();
                self.cursor.handle(event, cursor_max);

                if let Event::Input(key_event) = event {
                    if key_event.kind == KeyEventKind::Press {
                        #[allow(clippy::single_match)]
                        match key_event.code {
                            KeyCode::Enter => {
                                let asset = &assets[self.cursor.current];
                                self.asset = Some(asset.clone());
                                self.show_asset_popup = false;
                                // update form
                                *self.form.get_text_mut(FormItem::AssetType) =
                                    format!("{}", asset.r#type);
                                *self.form.get_currency_mut(FormItem::Amount).expect(
                                    "currency not found in this input entry, please check idx",
                                ) = Some(asset.r#type.symbol.clone());
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                // Assets not loaded yet
            }
        } else {
            unreachable!()
        }

        if self.address_book.is_some() || self.show_asset_popup {
            result.esc_ignores = 1;
        }

        if event.is_key_pressed(KeyCode::Esc) {
            self.address_book = None;
            self.show_asset_popup = false;
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
        shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf);

        if let Some(address_book) = &self.address_book {
            Popup {
                bg_color: Some(Color::Blue),
            }
            .render(area, buf);

            let inner_area = Popup::inner_area(area);
            let block = Block::bordered().title("Address Book");
            let block_inner_area = block.inner(inner_area);
            block.render(inner_area, buf);

            FilterSelect {
                full_list: address_book.list(),
                cursor: &self.cursor,
                search_string: &self.search_string,
                focus: shared_state.focus == Focus::Main,
            }
            .render(block_inner_area, buf);
        }

        if self.show_asset_popup {
            Popup {
                bg_color: Some(Color::Blue),
            }
            .render(area, buf);

            let inner_area = Popup::inner_area(area);
            let block = Block::bordered().title("Assets");
            let block_inner_area = block.inner(inner_area);
            block.render(inner_area, buf);

            if let Some(assets) = shared_state.assets.as_ref() {
                FilterSelect {
                    full_list: assets,
                    cursor: &self.cursor,
                    search_string: &self.search_string,
                    focus: shared_state.focus == Focus::Main,
                }
                .render(block_inner_area, buf);
            } else {
                "Loading assets..".render(block_inner_area, buf);
            }
        }

        area
    }
}
