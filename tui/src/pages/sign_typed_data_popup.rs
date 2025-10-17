use std::sync::mpsc;

use alloy::{
    dyn_abi::TypedData,
    primitives::{Address, B256},
    signers::{Signature, Signer},
};

use gm_ratatui_extra::{
    act::Act,
    button::Button,
    extensions::{RectExt, RenderTextWrapped, ThemedWidget},
    popup::{Popup, PopupWidget},
    text_interactive::TextInteractive,
    thematize::Thematize,
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};
use serde_json::Value;
use tokio::task::JoinHandle;

use crate::{
    app::SharedState, error::FmtError, post_handle_event::PostHandleEventActions, theme::Theme,
    AppEvent,
};
use gm_utils::{account::AccountManager, serde::SerdeResponseParse};

fn spawn_sign_thread(
    digest: B256,
    tr: &mpsc::Sender<AppEvent>,
    shared_state: &SharedState,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let signer_account = shared_state.try_current_account()?;

    Ok(tokio::spawn(async move {
        let _ = match run(digest, signer_account).await {
            Ok(sig) => tr.send(AppEvent::SignResult(signer_account, sig)),
            Err(err) => tr.send(AppEvent::SignError(err.fmt_err("SignError"))),
        };

        async fn run(digest: B256, signer_account: Address) -> crate::Result<Signature> {
            let wallet = AccountManager::load_wallet(signer_account)?;
            Ok(wallet.sign_hash(&digest).await?)
        }
    }))
}

#[derive(Debug, Default)]
enum SignStatus {
    #[default]
    Idle,
    Signing,
    Done,
    Failed,
}

#[derive(Debug)]
pub struct SignTypedDataPopup {
    typed_data_json: Value,
    display: TextInteractive,
    popup: Popup,
    cancel_button: Button,
    confirm_button: Button,
    is_confirm_focused: bool,
    status: SignStatus,
    sign_thread: Option<JoinHandle<()>>,
}

impl Default for SignTypedDataPopup {
    fn default() -> Self {
        Self::new()
    }
}

impl PopupWidget for SignTypedDataPopup {
    fn get_popup(&self) -> &Popup {
        &self.popup
    }

    fn get_popup_mut(&mut self) -> &mut Popup {
        &mut self.popup
    }

    fn open(&mut self) {
        self.popup.open();
        self.is_confirm_focused = false;
    }
}

impl SignTypedDataPopup {
    pub fn new() -> Self {
        Self {
            typed_data_json: Value::Null,
            display: TextInteractive::default(),
            popup: Popup::default().with_title("Sign EIP-712 Typed Data"),
            cancel_button: Button::new("Cancel"),
            confirm_button: Button::new("Confirm"),
            is_confirm_focused: true,
            status: SignStatus::Idle,
            sign_thread: None,
        }
    }

    pub fn set_typed_data(&mut self, v: Value) -> crate::Result<()> {
        if v.get("types").is_none() {
            return Err(crate::Error::TypedDataMissingField("types".to_string()));
        }
        if v.get("domain").is_none() {
            return Err(crate::Error::TypedDataMissingField("domain".to_string()));
        }
        if v.get("primaryType").is_none() {
            return Err(crate::Error::TypedDataMissingField(
                "primaryType".to_string(),
            ));
        }
        if v.get("message").is_none() {
            return Err(crate::Error::TypedDataMissingField("message".to_string()));
        }

        self.display.set_text(
            match serde_json::to_string_pretty(&v) {
                Ok(s) => format!("EIP-712 Typed Data:\n\n{s}\n\n"),
                Err(_) => format!("EIP-712 Typed Data (unprintable):\n\n{v}\n\n"),
            },
            true,
        );
        self.typed_data_json = v;
        self.reset();
        Ok(())
    }

    fn reset(&mut self) {
        self.is_confirm_focused = false;
        self.status = SignStatus::Idle;
        if let Some(thread) = self.sign_thread.take() {
            thread.abort();
        }
    }

    pub fn handle_event<F1, F3, F4>(
        &mut self,
        (event, popup_area, tr, ss): (&AppEvent, Rect, &mpsc::Sender<AppEvent>, &SharedState),
        mut on_signature: F1,
        mut on_cancel: F3,
        mut on_esc: F4,
    ) -> crate::Result<PostHandleEventActions>
    where
        F1: FnMut(&Signature) -> crate::Result<()>,
        F3: FnMut() -> crate::Result<()>,
        F4: FnMut() -> crate::Result<()>,
    {
        let mut result = PostHandleEventActions::default();

        if self.is_open() {
            let body_area = self.body_area(popup_area);

            self.display
                .handle_event(event.input_event(), body_area, &mut result);

            match event {
                AppEvent::Input(input_event) => match input_event {
                    Event::Key(key_event) => {
                        if key_event.kind == KeyEventKind::Press {
                            match self.status {
                                SignStatus::Idle => match key_event.code {
                                    KeyCode::Left => {
                                        self.is_confirm_focused = false;
                                    }
                                    KeyCode::Right => {
                                        self.is_confirm_focused = true;
                                    }
                                    KeyCode::Enter => {
                                        if self.is_confirm_focused {
                                            let typed_data = (&self.typed_data_json)
                                                .serde_parse_custom::<TypedData>()?;
                                            let digest = typed_data
                                                .eip712_signing_hash()
                                                .map_err(crate::Error::Eip712Error)?;
                                            self.status = SignStatus::Signing;
                                            self.sign_thread =
                                                Some(spawn_sign_thread(digest, tr, ss)?);
                                        } else {
                                            self.close();
                                            on_cancel()?;
                                        }
                                    }
                                    KeyCode::Esc => {
                                        self.close();
                                        on_esc()?;
                                    }
                                    _ => {}
                                },
                                SignStatus::Signing => {}
                                SignStatus::Done | SignStatus::Failed => {
                                    if key_event.code == KeyCode::Esc {
                                        self.close();
                                        on_esc()?;
                                    }
                                }
                            }
                        }
                    }
                    Event::Mouse(_mouse_event) => {}
                    _ => {}
                },
                AppEvent::SignResult(_, signature) => {
                    on_signature(signature)?;
                    self.status = SignStatus::Done;

                    if let Some(thread) = self.sign_thread.take() {
                        thread.abort();
                    }
                }
                AppEvent::SignError(_) => {
                    self.status = SignStatus::Failed;
                }
                _ => {}
            }
            result.ignore_esc()
        }
        Ok(result)
    }

    pub fn render(&self, popup_area: Rect, buf: &mut Buffer, theme: &Theme)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            self.popup.render(popup_area, buf, &theme);

            let [text_area, button_area] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(3)])
                    .areas(self.body_area(popup_area));

            self.display.render(text_area, buf, &theme);

            let [left_area, right_area] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(button_area);

            match self.status {
                SignStatus::Idle => {
                    self.cancel_button
                        .render(left_area, buf, !self.is_confirm_focused, &theme);

                    self.confirm_button
                        .render(right_area, buf, !self.is_confirm_focused, &theme);
                }
                SignStatus::Signing => {
                    "Signing data...".render(button_area.margin_top(1), buf);
                }
                SignStatus::Done => {
                    ["Signature is done.", "Press ESC to close"]
                        .render_wrapped(button_area.margin_top(1), buf);
                }
                SignStatus::Failed => {
                    ["Signing failed.", "Press ESC to close"]
                        .render_wrapped(button_area.margin_top(1), buf);
                }
            }
        }
    }
}
