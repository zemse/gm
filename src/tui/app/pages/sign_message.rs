use std::sync::{atomic::AtomicBool, mpsc, Arc};

use alloy::signers::SignerSync;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use strum::EnumIter;

use crate::{
    actions::account::load_wallet,
    tui::{
        app::{
            widgets::form::{Form, FormItemIndex, FormWidget},
            SharedState,
        },
        events::Event,
        traits::{Component, HandleResult},
    },
};

#[derive(EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    Message,
    SignMessageButton,
    Signature,
}

pub struct SignMessagePage {
    form: Form<FormItem>,
}
impl FormItemIndex for FormItem {
    fn index(self) -> usize {
        self as usize
    }
}
impl From<FormItem> for FormWidget {
    fn from(value: FormItem) -> Self {
        match value {
            FormItem::Heading => FormWidget::Heading("Sign a Message"),
            FormItem::Message => FormWidget::InputBox {
                label: "Message",
                text: String::new(),
                empty_text: Some("Type message to sign"),
                currency: None,
            },
            FormItem::SignMessageButton => FormWidget::Button {
                label: "Sign Message",
            },
            FormItem::Signature => FormWidget::DisplayText(String::new()),
        }
    }
}

impl Default for SignMessagePage {
    fn default() -> Self {
        Self {
            form: Form::init(|_| {}),
        }
    }
}

impl Component for SignMessagePage {
    fn set_focus(&mut self, focus: bool) {
        self.form.set_form_focus(focus);
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        self.form.handle_event(event, |item, form| {
            if item == FormItem::SignMessageButton && form.get_text(FormItem::Signature).is_empty()
            {
                let message = form.get_text(FormItem::Message);

                let wallet_address = shared_state
                    .current_account
                    .ok_or(crate::Error::CurrentAccountNotSet)?;
                let wallet = load_wallet(wallet_address)?;
                let signature = wallet.sign_message_sync(message.as_bytes())?;
                *form.get_text_mut(FormItem::Signature) = format!("Signature:\n{}", signature);
            }
            Ok(())
        })?;
        Ok(HandleResult::default())
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, _: &SharedState) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf);

        area
    }
}
