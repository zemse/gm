use std::sync::mpsc;

use alloy::signers::SignerSync;
use gm_ratatui_extra::{
    button::Button,
    form::{Form, FormItemIndex, FormWidget},
    input_box_owned::InputBoxOwned,
};
use ratatui::{buffer::Buffer, layout::Rect};
use strum::{Display, EnumIter};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState, post_handle_event::PostHandleEventActions, traits::Component, AppEvent,
};
use gm_utils::account::AccountManager;

#[derive(Debug, Display, EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    Message,
    SignMessageButton,
    LineBreak,
    Signature,
}

#[derive(Debug)]
pub struct SignMessagePage {
    form: Form<FormItem, crate::Error>,
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
            FormItem::Heading => FormWidget::Heading("Sign a Message"),
            FormItem::Message => FormWidget::InputBox {
                widget: InputBoxOwned::new("Message").with_empty_text("Type message to sign"),
            },
            FormItem::SignMessageButton => FormWidget::Button {
                widget: Button::new("Sign Message"),
            },
            FormItem::LineBreak => FormWidget::LineBreak,
            FormItem::Signature => FormWidget::DisplayText(String::new()),
        };
        Ok(widget)
    }
}

impl SignMessagePage {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            form: Form::init(|_| Ok(()))?,
        })
    }
}

impl Component for SignMessagePage {
    fn set_focus(&mut self, focus: bool) {
        self.form.set_form_focus(focus);
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _popup_area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        self.form.handle_event(
            event.widget_event().as_ref(),
            area,
            |_, _| Ok(()),
            |item, form| {
                if item == FormItem::SignMessageButton
                    && form.get_text(FormItem::Signature).is_empty()
                {
                    let message = form.get_text(FormItem::Message);

                    let wallet_address = shared_state.try_current_account()?;
                    let wallet = AccountManager::load_wallet(&wallet_address)?;
                    let signature = wallet.sign_message_sync(message.as_bytes())?;
                    form.set_text(FormItem::Signature, format!("Signature:\n{signature}"));
                }
                Ok(())
            },
        )
    }

    fn render_component(
        &self,
        area: Rect,
        popup_area: Rect,
        buf: &mut Buffer,
        ss: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, popup_area, buf, &ss.theme);

        area
    }
}
