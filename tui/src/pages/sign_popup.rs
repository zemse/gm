use std::sync::mpsc;

use alloy::{hex, primitives::Address, signers::Signature};

use gm_ratatui_extra::{
    act::Act,
    confirm_popup::{ConfirmPopup, ConfirmResult},
    extensions::{CustomRender, ThemedWidget},
    popup::{Popup, PopupWidget},
    text_popup::TextPopup,
    text_scroll::TextScroll,
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
    /// Step 1 - User confirms or rejects signing
    Confirm { confirm_popup: ConfirmPopup },

    /// Step 2 - Signing in progress
    Signing {
        signer_account: Address,
        text_popup: TextPopup,
        sign_thread: JoinHandle<()>,
        receiver: oneshot::Receiver<gm_utils::Result<Signature>>,
    },

    /// Step 3 - Signing is done or failed
    Result {
        popup: Popup,
        signature: Option<Signature>,
    },
}

impl Default for SignPopup {
    fn default() -> Self {
        Self::confirm_screen()
    }
}

impl SignPopup {
    /// Create a new SignPopup with the given hex message to sign
    pub fn new_with_message_hex(msg_hex: &str) -> crate::Result<Self> {
        let mut popup = Self::confirm_screen();
        popup.set_msg_hex(msg_hex)?;
        Ok(popup)
    }

    /// Create a new SignPopup with the given utf8 message to sign
    pub fn new_with_message_utf8(msg: String) -> Self {
        let mut popup = Self::confirm_screen();
        popup.set_msg_utf8(msg);
        popup
    }

    fn confirm_screen() -> Self {
        Self::Confirm {
            confirm_popup: ConfirmPopup::new("Sign Message", String::new(), "Sign", "Cancel", true),
        }
    }

    fn signing_screen(
        signer_account: Address,
        message: TextScroll,
        sign_thread: JoinHandle<()>,
        receiver: oneshot::Receiver<gm_utils::Result<Signature>>,
    ) -> Self {
        Self::Signing {
            signer_account,
            // TODO enable initialising TextPopup with TextScroll
            text_popup: TextPopup::default()
                .with_title("Signing Message")
                .with_text(message.text)
                .with_break_words(true),
            sign_thread,
            receiver,
        }
    }

    fn result_screen(signature: Option<Signature>) -> Self {
        Self::Result {
            popup: Popup::default().with_title(if signature.is_some() {
                "Sign Message Result"
            } else {
                "Sign Message Failed"
            }),
            signature,
        }
    }

    pub fn is_open(&self) -> bool {
        match self {
            SignPopup::Confirm { confirm_popup } => confirm_popup.is_open(),
            SignPopup::Signing { text_popup, .. } => text_popup.is_open(),
            SignPopup::Result { popup, .. } => popup.is_open(),
        }
    }

    #[track_caller]
    pub fn open(&mut self) {
        match self {
            SignPopup::Confirm { confirm_popup } => {
                confirm_popup.open();
            }
            SignPopup::Signing { .. } | SignPopup::Result { .. } => {
                // The code that calls open() should prepare a fresh "Confirm" and then open.
                unreachable!("Cannot open sign_popup in this state")
            }
        }
    }

    pub fn close(&mut self) {
        match self {
            SignPopup::Confirm { confirm_popup } => {
                confirm_popup.close();
            }
            SignPopup::Signing { text_popup, .. } => {
                text_popup.close();
            }
            SignPopup::Result { popup, .. } => {
                popup.close();
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
            SignPopup::Confirm { confirm_popup } => {
                let utf8_str = hex::decode(msg_hex)
                    .map_err(crate::Error::FromHexError)
                    .and_then(|bytes| {
                        String::from_utf8(bytes).map_err(crate::Error::FromUtf8Error)
                    })?;

                *confirm_popup.text_mut() = utf8_str;
            }
            SignPopup::Signing { .. } | SignPopup::Result { .. } => {
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
            SignPopup::Confirm { confirm_popup } => {
                *confirm_popup.text_mut() = msg;
            }
            SignPopup::Signing { .. } | SignPopup::Result { .. } => {
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

            match self {
                SignPopup::Confirm { confirm_popup } => {
                    match confirm_popup.handle_event(event.input_event(), popup_area, actions)? {
                        Some(ConfirmResult::Confirmed) => {
                            let text_scroll = confirm_popup.into_text_scroll();

                            let signer_account = ss.try_current_account()?;
                            let data = {
                                let message = text_scroll.text.clone();
                                match hex::decode(&message) {
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
                            *self = Self::signing_screen(signer_account, text_scroll, thread, rc);
                        }
                        Some(ConfirmResult::Canceled) => {
                            self.close();
                            result = Some(SignPopupEvent::Rejected);
                        }
                        None => {}
                    }
                }
                SignPopup::Signing {
                    sign_thread,
                    receiver,
                    signer_account,
                    ..
                } => {
                    if let Ok(sign_result) = receiver.try_recv() {
                        match sign_result {
                            Ok(signature) => {
                                result = Some(SignPopupEvent::Signed(*signer_account, signature));
                                sign_thread.abort();

                                *self = Self::result_screen(Some(signature));
                            }
                            Err(err) => {
                                sign_thread.abort();
                                self.close();
                                return Err(err.into());
                            }
                        }
                    }
                }
                SignPopup::Result { popup, .. } => {
                    popup.handle_event(event.input_event(), actions);
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
                SignPopup::Confirm { confirm_popup } => {
                    confirm_popup.render(popup_area, buf, theme);
                }
                SignPopup::Signing { text_popup, .. } => {
                    // TODO change this into a simple popup without text
                    text_popup.render(popup_area, buf, theme);

                    Span::raw("Signing message...")
                        .style(theme.style_dim())
                        .render(text_popup.body_area(popup_area), buf);
                }
                SignPopup::Result { popup, .. } => {
                    popup.render(popup_area, buf, theme);

                    ["Signature is done.", "Press ESC to close"].render(
                        popup.body_area(popup_area),
                        buf,
                        (),
                    );
                }
            }
        }
    }
}
