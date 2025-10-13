use std::sync::mpsc;

use alloy::{hex, primitives::Address, signers::Signature};
use gm_ratatui_extra::{
    button::Button,
    confirm_popup::{ConfirmPopup, ConfirmResult},
    extensions::RenderTextWrapped,
    form::{Form, FormEvent, FormItemIndex, FormWidget},
    input_box::InputBox,
};
use gm_utils::etherscan::publish_signature_to_etherscan;
use ratatui::{buffer::Buffer, layout::Rect};
use strum::{Display, EnumIter};
use tokio::{sync::oneshot, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState,
    pages::sign_popup::{SignPopup, SignPopupEvent},
    post_handle_event::PostHandleEventActions,
    traits::Component,
    AppEvent,
};

#[derive(Debug, Display, EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    InfoText,
    MessageInput,
    SignMessageButton,
}

#[derive(Debug)]
pub enum SignMessagePage {
    /// Step 1 - Sign a message
    SignForm {
        form: Form<FormItem, crate::Error>,
        sign_popup: SignPopup,
    },

    /// Step 2 - Publish to Etherscan optionally
    PublishToEtherscan {
        address: Address,
        message: String,
        signature: Signature,
        confirm_popup: ConfirmPopup,
        publish_thread: Option<JoinHandle<()>>,
        result_receiver: Option<oneshot::Receiver<gm_utils::Result<String>>>,
    },

    /// Step 3 - Show result of publishing
    Result {
        signature: Signature,
        etherscan_url: Option<String>,
    },
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
            FormItem::InfoText => FormWidget::StaticText(
                "You can also publish this signature to a public URL on Etherscan for free. This can be used to prove to someone that you own this address using a custom message.",
            ),
            FormItem::MessageInput => FormWidget::InputBox {
                widget: InputBox::new("Message").with_empty_text("Type message to sign"),
            },
            FormItem::SignMessageButton => FormWidget::Button {
                widget: Button::new("Sign Message"),
            },
        };
        Ok(widget)
    }
}

impl SignMessagePage {
    pub fn new() -> crate::Result<Self> {
        Ok(Self::SignForm {
            form: Form::init(|_| Ok(()))?,
            sign_popup: SignPopup::default(),
        })
    }

    fn show_publish_to_etherscan_screen(
        &mut self,
        address: Address,
        message: String,
        signature: Signature,
    ) {
        *self = Self::PublishToEtherscan {
            address,
            message,
            signature,
            confirm_popup: ConfirmPopup::new(
                "Publish to Etherscan?",
                "Your signature will be published to etherscan for free and a sharable link will be generated.".to_string(),
                "Publish",
                "Skip",
                true,
            ).open_already(),
            publish_thread: None,
            result_receiver: None,
        }
    }
}

impl Component for SignMessagePage {
    fn set_focus(&mut self, focus: bool) {
        match self {
            Self::SignForm { form, .. } => {
                form.set_form_focus(focus);
            }
            Self::PublishToEtherscan { .. } => {}
            Self::Result { .. } => {}
        }
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        popup_area: Rect,
        transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let mut actions = PostHandleEventActions::default();

        match self {
            Self::SignForm { form, sign_popup } => {
                if sign_popup.is_open() {
                    if let Some(sign_popup_event) = sign_popup.handle_event(
                        (event, popup_area, transmitter, shared_state),
                        &mut actions,
                    )? {
                        match sign_popup_event {
                            SignPopupEvent::Signed(address, signature) => {
                                let message = form.get_text(FormItem::MessageInput).to_string();
                                self.show_publish_to_etherscan_screen(address, message, signature);
                            }
                            SignPopupEvent::Rejected
                            | SignPopupEvent::EscapedBeforeSigning
                            | SignPopupEvent::EscapedAfterSigning => sign_popup.close(),
                        }
                    }
                } else {
                    // Handle form events
                    if let Some(FormEvent::ButtonPressed(item)) = form.handle_event(
                        event.widget_event().as_ref(),
                        area,
                        popup_area,
                        &mut actions,
                    )? {
                        if item == FormItem::SignMessageButton {
                            let message = form.get_text(FormItem::MessageInput);
                            *sign_popup = SignPopup::new_with_message_utf8(message.to_string());
                            sign_popup.open();
                        }
                    }
                }
            }
            Self::PublishToEtherscan {
                address,
                message,
                signature,
                confirm_popup,
                publish_thread,
                result_receiver,
            } => {
                if let Some(ConfirmResult::Confirmed) =
                    confirm_popup.handle_event(event.input_event(), popup_area, &mut actions)?
                {
                    let address = *address;
                    let message = message.clone();
                    let signature = *signature;
                    let (tr, rc) = oneshot::channel::<gm_utils::Result<String>>();
                    *publish_thread = Some(tokio::spawn(async move {
                        let _ = tr.send(
                            publish_signature_to_etherscan(address, message, signature).await,
                        );
                    }));
                    *result_receiver = Some(rc);
                }

                if let Some(rc) = result_receiver {
                    if let Ok(result) = rc.try_recv() {
                        match result {
                            Ok(url) => {
                                let _ = open::that(&url);
                                *self = Self::Result {
                                    signature: *signature,
                                    etherscan_url: Some(url),
                                };
                            }
                            Err(err) => {
                                *self = Self::Result {
                                    signature: *signature,
                                    etherscan_url: None,
                                };
                                return Err(err.into());
                            }
                        }
                    }
                }
            }

            // No need to handle anything
            // TODO handle copy events
            Self::Result { .. } => {}
        }

        Ok(actions)
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
        match self {
            Self::SignForm { form, sign_popup } => {
                form.render(area, popup_area, buf, &ss.theme);

                sign_popup.render(popup_area, buf, &ss.theme);
            }
            Self::PublishToEtherscan {
                signature,
                confirm_popup,
                ..
            } => {
                format!("Signature: {}", hex::encode_prefixed(signature.as_bytes()))
                    .render_wrapped(area, buf);

                confirm_popup.render(popup_area, buf, &ss.theme);
            }
            Self::Result {
                signature,
                etherscan_url,
            } => {
                let mut lines = vec![format!(
                    "Signature: {}",
                    hex::encode_prefixed(signature.as_bytes())
                )];

                lines.push(String::new());

                if let Some(url) = etherscan_url {
                    lines.push(format!("Etherscan URL: {url}"));
                } else {
                    lines.push("Failed to publish to Etherscan.".to_string());
                }

                lines.render_wrapped(area, buf);
            }
        }

        area
    }
}
