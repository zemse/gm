use crate::app::SharedState;
use crate::pages::token::TokenPage;
use crate::pages::Page;
use crate::post_handle_event::PostHandleEventActions;
use crate::traits::Component;
use crate::AppEvent;
use alloy::primitives::Address;

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
use std::sync::mpsc::Sender;
use strum::Display;
use strum_macros::EnumIter;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Display, EnumIter, PartialEq)]
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
                widget: InputBox::new("Name"),
            },
            FormItem::Symbol => FormWidget::InputBox {
                widget: InputBox::new("Symbol"),
            },
            FormItem::Decimals => FormWidget::InputBox {
                widget: InputBox::new("Decimals"),
            },
            FormItem::ContractAddress => FormWidget::InputBox {
                widget: InputBox::new("Contract Address"),
            },
            FormItem::SaveButton => FormWidget::Button {
                widget: Button::new("Save"),
            },
            FormItem::RemoveButton => FormWidget::Button {
                widget: Button::new("Remove"),
            },
            FormItem::ErrorText => FormWidget::ErrorText(String::new()),
        };
        Ok(widget)
    }
}

#[derive(Debug)]
pub struct TokenCreatePage {
    pub is_new: bool,
    pub form: Form<FormItem, crate::Error>,
    pub token: Token,
    pub token_index: usize,
    pub network: Network,
    pub network_index: usize,
    pub remove_popup: ConfirmPopup,
}
impl TokenCreatePage {
    pub fn new(
        is_new: bool,
        token_index: usize,
        network_index: usize,
        network: Network,
    ) -> crate::Result<Self> {
        let token = network.tokens.get(token_index).cloned().unwrap_or_default();
        Ok(Self {
            is_new,
            form: Form::init(|form| {
                form.set_text(FormItem::Name, token.name.clone());
                form.set_text(FormItem::Symbol, token.symbol.clone());
                form.set_text(FormItem::Decimals, token.decimals.to_string());
                form.set_text(
                    FormItem::ContractAddress,
                    token.contract_address.to_string(),
                );
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
                true,
            ),
            token,
            network,
            network_index,
        })
    }
    pub fn token(form: &Form<FormItem, crate::Error>) -> Token {
        Token {
            name: form.get_text(FormItem::Name).to_string(),
            symbol: form.get_text(FormItem::Symbol).to_string(),
            decimals: form
                .get_text(FormItem::Decimals)
                .parse()
                .unwrap_or_default(),
            contract_address: form
                .get_text(FormItem::ContractAddress)
                .parse::<Address>()
                .unwrap_or_default(),
        }
    }
}
impl Component for TokenCreatePage {
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
                    .handle_event(event.input_event(), area, &mut actions)?
            {
                if self.network.tokens.get(self.token_index).is_some() {
                    self.network.tokens.remove(self.token_index);
                    let mut store = NetworkStore::load()?;
                    store.networks[self.network_index] = self.network.clone();
                    store.save()?;
                }
                actions.page_pop();
                actions.page_insert(Page::Token(TokenPage::new(
                    self.network_index,
                    self.network.clone(),
                )?));
                actions.reload();
            }
        }

        if let Some(FormEvent::ButtonPressed(label)) = self.form.handle_event(
            event.widget_event().as_ref(),
            area,
            popup_area,
            &mut actions,
        )? {
            if label == FormItem::SaveButton {
                let token = Self::token(&self.form);
                if self.network.tokens.get(self.token_index).is_some() {
                    self.network.tokens[self.token_index] = token;
                } else {
                    self.network.tokens.push(token);
                }

                let mut config = NetworkStore::load()?;
                config.networks[self.network_index] = self.network.clone();
                config.save()?;

                actions.page_pop();
                actions.page_insert(Page::Token(TokenPage::new(
                    self.network_index,
                    self.network.clone(),
                )?));
                actions.reload();
            }
            if label == FormItem::RemoveButton {
                self.remove_popup.open();
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
