use std::{mem, sync::mpsc};

use alloy::{hex, primitives::Address, signers::Signature};

use gm_ratatui_extra::{
    act::Act,
    confirm_popup::{ConfirmPopup, ConfirmResult},
    extensions::{RenderTextWrapped, ThemedWidget},
    popup::PopupWidget,
    text_interactive::TextInteractive,
    text_popup::TextPopup,
    thematize::Thematize,
};
use gm_utils::account::AccountManager;
use ratatui::{buffer::Buffer, layout::Rect, text::Span, widgets::Widget};
use tokio::{sync::oneshot, task::JoinHandle};

use crate::{app::SharedState, post_handle_event::PostHandleEventActions, theme::Theme, AppEvent};

pub enum SignPopupEvent {
    Signed(Address, Signature),
    Rejected,
    EscapedBeforeSigning,
    EscapedAfterSigning,
}

#[derive(Debug)]
pub enum SignPopup {
    Closed,

    /// Step 1 - User confirms or rejects signing
    Prompt {
        confirm_popup: ConfirmPopup,
    },

    /// Step 2 - Signing in progress
    Signing {
        signer_account: Address,
        text_popup: TextPopup,
        sign_thread: JoinHandle<()>,
        receiver: oneshot::Receiver<gm_utils::Result<Signature>>,
    },

    /// Step 3 - Signing is done or failed
    Done {
        text_popup: TextPopup,
        signature: Option<Signature>,
    },
}

impl PopupWidget for SignPopup {
    #[track_caller]
    fn get_popup_inner(&self) -> &dyn PopupWidget {
        match self {
            SignPopup::Closed => unreachable!("SignPopup::get_popup_inner Closed"),
            SignPopup::Prompt { confirm_popup, .. } => confirm_popup as &dyn PopupWidget,
            SignPopup::Signing { text_popup, .. } => text_popup as &dyn PopupWidget,
            SignPopup::Done { text_popup, .. } => text_popup as &dyn PopupWidget,
        }
    }

    #[track_caller]
    fn get_popup_inner_mut(&mut self) -> &mut dyn PopupWidget {
        match self {
            SignPopup::Closed => unreachable!("SignPopup::get_popup_inner_mut Closed"),
            SignPopup::Prompt { confirm_popup, .. } => confirm_popup as &mut dyn PopupWidget,
            SignPopup::Signing { text_popup, .. } => text_popup as &mut dyn PopupWidget,
            SignPopup::Done { text_popup, .. } => text_popup as &mut dyn PopupWidget,
        }
    }

    fn is_open(&self) -> bool {
        if matches!(self, SignPopup::Closed) {
            false
        } else {
            self.get_popup_inner().is_open()
        }
    }

    #[track_caller]
    fn open(&mut self) {
        if matches!(self, SignPopup::Closed) {
            unreachable!(
                "SignPopup::open called when Closed, use SignPopup::new_with_message_* instead"
            );
        } else {
            self.get_popup_inner_mut().open();
        }
    }
}

impl SignPopup {
    /// Create a new SignPopup with the given hex message to sign
    pub fn new_with_message_hex(msg_hex: &str) -> crate::Result<Self> {
        let mut popup = Self::prompt_screen();
        popup.set_msg_hex(msg_hex)?;
        Ok(popup)
    }

    /// Create a new SignPopup with the given utf8 message to sign
    pub fn new_with_message_utf8(msg: String) -> Self {
        let mut popup = Self::prompt_screen();
        popup.set_msg_utf8(msg);
        popup
    }

    fn prompt_screen() -> Self {
        Self::Prompt {
            confirm_popup: ConfirmPopup::new("Sign", "Cancel", true).with_title("Sign Message"),
        }
    }

    fn signing_screen(
        signer_account: Address,
        message: TextInteractive,
        sign_thread: JoinHandle<()>,
        receiver: oneshot::Receiver<gm_utils::Result<Signature>>,
    ) -> Self {
        Self::Signing {
            signer_account,
            // TODO enable initialising TextPopup with TextScroll
            text_popup: TextPopup::default()
                .with_title("Signing Message")
                .with_text(message.into_text()),
            sign_thread,
            receiver,
        }
    }

    fn done_screen(signature: Option<Signature>) -> Self {
        Self::Done {
            text_popup: TextPopup::default()
                .with_title(if signature.is_some() {
                    "Sign Message Result"
                } else {
                    "Sign Message Failed"
                })
                .with_text(
                    signature
                        .map(|signature| signature.to_string())
                        .unwrap_or_else(|| "Failed to sign the message.".to_string()),
                ),
            signature,
        }
    }

    pub fn is_open(&self) -> bool {
        match self {
            SignPopup::Closed => false,
            SignPopup::Prompt { confirm_popup } => confirm_popup.is_open(),
            SignPopup::Signing { text_popup, .. } => text_popup.is_open(),
            SignPopup::Done { text_popup, .. } => text_popup.is_open(),
        }
    }

    #[track_caller]
    pub fn open(&mut self) {
        match self {
            SignPopup::Closed => unreachable!("Null"),
            SignPopup::Prompt { confirm_popup } => {
                confirm_popup.open();
            }
            SignPopup::Signing { .. } | SignPopup::Done { .. } => {
                // The code that calls open() should prepare a fresh "Confirm" and then open.
                unreachable!("Cannot open sign_popup in this state")
            }
        }
    }

    pub fn close(&mut self) {
        match self {
            SignPopup::Closed => unreachable!("Null"),
            SignPopup::Prompt { confirm_popup } => {
                confirm_popup.close();
            }
            SignPopup::Signing { text_popup, .. } => {
                text_popup.close();
            }
            SignPopup::Done { text_popup, .. } => {
                text_popup.close();
            }
        }
    }

    /// Set the message to sign, given in hex format.
    ///
    /// Note: This is marked private intentionally because we reuse an existing SignPopup by updating
    /// text on it because of internal state issue. Specifically, the button might have hover_focus true
    /// on the "cancel" button, so if we reuse the same SignPopup instance, the popup that gets opened
    /// has the cursor on "confirm" so it is focused, and "cancel" is also focused due to previous hover.
    #[track_caller]
    fn set_msg_hex(&mut self, msg_hex: &str) -> crate::Result<()> {
        match self {
            SignPopup::Closed => {
                *self = SignPopup::prompt_screen();
                self.set_msg_hex(msg_hex)?;
            }
            SignPopup::Prompt { confirm_popup } => {
                let utf8_str = hex::decode(msg_hex)
                    .map_err(crate::Error::FromHexError)
                    .and_then(|bytes| {
                        String::from_utf8(bytes).map_err(crate::Error::FromUtf8Error)
                    })?;

                confirm_popup.set_text(utf8_str, true);
            }
            SignPopup::Signing { .. } | SignPopup::Done { .. } => {
                unreachable!("Cannot change message data in this state")
            }
        }

        Ok(())
    }

    /// Set the message to sign, given in utf8 format.
    ///
    /// Note: <similar to the comment in set_msg_hex>
    #[track_caller]
    fn set_msg_utf8(&mut self, msg: String) {
        match self {
            SignPopup::Closed => {
                *self = SignPopup::prompt_screen();
                self.set_msg_utf8(msg);
            }
            SignPopup::Prompt { confirm_popup } => {
                confirm_popup.set_text(msg, true);
            }
            SignPopup::Signing { .. } | SignPopup::Done { .. } => {
                unreachable!("Cannot change message data in this state")
            }
        }
    }

    pub fn handle_event(
        &mut self,
        (event, popup_area, _tr, ss): (&AppEvent, Rect, &mpsc::Sender<AppEvent>, &SharedState),
        actions: &mut PostHandleEventActions,
    ) -> crate::Result<Option<SignPopupEvent>> {
        let mut result = None;

        if self.is_open() {
            actions.ignore_esc();

            let self_owned = mem::replace(self, Self::Closed);

            *self = match self_owned {
                SignPopup::Closed => self_owned, // Do nothing
                SignPopup::Prompt { mut confirm_popup } => {
                    match confirm_popup.handle_event(event.input_event(), popup_area, actions)? {
                        Some(ConfirmResult::Confirmed) => {
                            let text_scroll = confirm_popup.into_text();

                            let signer_account = ss.try_current_account()?;
                            let data = {
                                let message = text_scroll.text();
                                match hex::decode(message) {
                                    Ok(bytes) => bytes,
                                    Err(_) => message.as_bytes().to_vec(),
                                }
                            };

                            let (tr, rc) = oneshot::channel::<gm_utils::Result<Signature>>();
                            let thread = tokio::spawn(async move {
                                let _ = tr.send(
                                    AccountManager::sign_message_async(signer_account, data).await,
                                );
                            });

                            // Move to signing screen
                            SignPopup::signing_screen(signer_account, text_scroll, thread, rc)
                        }
                        Some(ConfirmResult::Canceled) => {
                            confirm_popup.close();
                            result = Some(SignPopupEvent::Rejected);
                            SignPopup::Prompt { confirm_popup }
                        }
                        None => SignPopup::Prompt { confirm_popup }, // Do nothing
                    }
                }
                SignPopup::Signing {
                    sign_thread,
                    mut receiver,
                    signer_account,
                    mut text_popup,
                } => {
                    text_popup.handle_event(event.input_event(), popup_area, actions);

                    if let Ok(sign_result) = receiver.try_recv() {
                        match sign_result {
                            Ok(signature) => {
                                result = Some(SignPopupEvent::Signed(signer_account, signature));
                                sign_thread.abort();

                                Self::done_screen(Some(signature))
                            }
                            Err(err) => {
                                sign_thread.abort();
                                self.close();
                                return Err(err.into());
                            }
                        }
                    } else {
                        SignPopup::Signing {
                            sign_thread,
                            receiver,
                            signer_account,
                            text_popup,
                        } // do nothing, still signing
                    }
                }
                SignPopup::Done {
                    mut text_popup,
                    signature,
                } => {
                    text_popup.handle_event(event.input_event(), popup_area, actions);

                    SignPopup::Done {
                        text_popup,
                        signature,
                    }
                }
            }
        }

        Ok(result)
    }

    pub fn render(&self, popup_area: Rect, buf: &mut Buffer, theme: &Theme)
    where
        Self: Sized,
    {
        if self.is_open() {
            match self {
                SignPopup::Closed => {}
                SignPopup::Prompt { confirm_popup } => {
                    confirm_popup.render(popup_area, buf, theme);
                }
                SignPopup::Signing { text_popup, .. } => {
                    // TODO change this into a simple popup without text
                    text_popup.render(popup_area, buf, theme);

                    Span::raw("Signing message...")
                        .style(theme.style_dim())
                        .render(text_popup.body_area(popup_area), buf);
                }
                SignPopup::Done { text_popup, .. } => {
                    text_popup.render(popup_area, buf, theme);

                    ["Signature is done.", "Press ESC to close"]
                        .render_wrapped(text_popup.body_area(popup_area), buf);
                }
            }
        }
    }
}
