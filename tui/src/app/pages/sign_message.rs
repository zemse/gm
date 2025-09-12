use std::sync::{atomic::AtomicBool, mpsc, Arc};

use alloy::signers::SignerSync;
use ratatui::{buffer::Buffer, layout::Rect};
use strum::EnumIter;

use crate::{
    app::{
        widgets::form::{Form, FormItemIndex, FormWidget},
        SharedState,
    },
    events::Event,
    traits::{Component, HandleResult},
};
use gm_utils::account::AccountManager;

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
impl TryFrom<FormItem> for FormWidget {
    type Error = crate::Error;
    fn try_from(value: FormItem) -> crate::Result<Self> {
        let widget = match value {
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
        event: &Event,
        _area: Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        self.form.handle_event(
            event,
            |_, _| Ok(()),
            |item, form| {
                if item == FormItem::SignMessageButton
                    && form.get_text(FormItem::Signature).is_empty()
                {
                    let message = form.get_text(FormItem::Message);

                    let wallet_address = shared_state.try_current_account()?;
                    let wallet = AccountManager::load_wallet(&wallet_address)?;
                    let signature = wallet.sign_message_sync(message.as_bytes())?;
                    *form.get_text_mut(FormItem::Signature) = format!("Signature:\n{signature}");
                }
                Ok(())
            },
        )?;
        Ok(HandleResult::default())
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, ss: &SharedState) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf, &ss.theme);

        area
    }
}
