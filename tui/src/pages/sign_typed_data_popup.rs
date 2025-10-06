use std::sync::mpsc;

use alloy::{
    dyn_abi::TypedData,
    primitives::{Address, B256},
    signers::{Signature, Signer},
};

use gm_ratatui_extra::{
    act::Act,
    button::Button,
    extensions::{CustomRender, RectExt, ThemedWidget},
    popup::Popup,
    text_scroll::TextScroll,
    thematize::Thematize,
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
};
use serde_json::Value;
use tokio::task::JoinHandle;

use crate::{app::SharedState, error::FmtError, theme::Theme, traits::Actions, AppEvent};
use gm_utils::{account::AccountManager, serde::SerdeResponseParse};

fn spawn_sign_thread(
    digest: B256,
    tr: &mpsc::Sender<AppEvent>,
    shared_state: &SharedState,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let sender_account = shared_state
        .current_account
        .ok_or(crate::Error::CurrentAccountNotSet)?;

    Ok(tokio::spawn(async move {
        let _ = match run(digest, sender_account).await {
            Ok(sig) => tr.send(AppEvent::SignResult(sig)),
            Err(err) => tr.send(AppEvent::SignError(err.fmt_err("SignError"))),
        };

        async fn run(digest: B256, sender_account: Address) -> crate::Result<Signature> {
            let wallet = AccountManager::load_wallet(&sender_account)?;
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
    display: TextScroll,
    open: bool,
    button_cursor: bool,
    status: SignStatus,
    sign_thread: Option<JoinHandle<()>>,
}

impl Default for SignTypedDataPopup {
    fn default() -> Self {
        Self::new()
    }
}

impl SignTypedDataPopup {
    pub fn new() -> Self {
        Self {
            typed_data_json: Value::Null,
            display: TextScroll::default(),
            open: false,
            button_cursor: false,
            status: SignStatus::Idle,
            sign_thread: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
        self.button_cursor = false;
    }

    pub fn close(&mut self) {
        self.open = false;
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

        self.display.text = match serde_json::to_string_pretty(&v) {
            Ok(s) => format!("EIP-712 Typed Data:\n\n{s}\n\n"),
            Err(_) => format!("EIP-712 Typed Data (unprintable):\n\n{v}\n\n"),
        };
        self.typed_data_json = v;
        self.reset();
        Ok(())
    }

    fn reset(&mut self) {
        self.button_cursor = false;
        self.status = SignStatus::Idle;
        if let Some(thread) = self.sign_thread.take() {
            thread.abort();
        }
    }

    pub fn handle_event<F1, F3, F4>(
        &mut self,
        (event, area, tr, ss): (&AppEvent, Rect, &mpsc::Sender<AppEvent>, &SharedState),
        mut on_signature: F1,
        mut on_cancel: F3,
        mut on_esc: F4,
    ) -> crate::Result<Actions>
    where
        F1: FnMut(&Signature) -> crate::Result<()>,
        F3: FnMut() -> crate::Result<()>,
        F4: FnMut() -> crate::Result<()>,
    {
        let mut result = Actions::default();

        if self.is_open() {
            let area = Popup::inner_area(area).block_inner().margin_down(3);

            self.display.handle_event(event.key_event(), area);

            match event {
                AppEvent::Input(input_event) => match input_event {
                    Event::Key(key_event) => {
                        if key_event.kind == KeyEventKind::Press {
                            match self.status {
                                SignStatus::Idle => match key_event.code {
                                    KeyCode::Left => {
                                        self.button_cursor = false;
                                    }
                                    KeyCode::Right => {
                                        self.button_cursor = true;
                                    }
                                    KeyCode::Enter => {
                                        if self.button_cursor {
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
                AppEvent::SignResult(signature) => {
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

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            Popup.render(area, buf, &theme);

            let inner_area = Popup::inner_area(area);
            let block = Block::bordered().title("Sign EIP-712 Typed Data");
            let block_inner_area = block.inner(inner_area);
            block.render(inner_area, buf);

            let [text_area, button_area] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(3)])
                    .areas(block_inner_area);

            self.display.render(text_area, buf, &theme);

            let [left_area, right_area] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(button_area);

            match self.status {
                SignStatus::Idle => {
                    Button {
                        focus: !self.button_cursor,
                        label: "Cancel",
                    }
                    .render(left_area, buf, &theme);

                    Button {
                        focus: self.button_cursor,
                        label: "Confirm",
                    }
                    .render(right_area, buf, &theme);
                }
                SignStatus::Signing => {
                    "Signing data...".render(button_area.margin_top(1), buf);
                }
                SignStatus::Done => {
                    ["Signature is done.", "Press ESC to close"].render(
                        button_area.margin_top(1),
                        buf,
                        (),
                    );
                }
                SignStatus::Failed => {
                    ["Signing failed.", "Press ESC to close"].render(
                        button_area.margin_top(1),
                        buf,
                        (),
                    );
                }
            }
        }
    }
}

// TODO there is a bug here that causes revert 0x815e1d64 from permit2, found while interacting with euler
// fn eip712_digest_from_json(typed_data: &Value) -> crate::Result<B256> {
//     let (_msg_type, message_value, _domain_type, domain_value) = eip712_to_dyn(typed_data)?;

//     let encoded_domain = domain_value.abi_encode();
//     let encoded_message = message_value.abi_encode();

//     let domain_separator = keccak256(&encoded_domain);
//     let message_hash = keccak256(&encoded_message);
//     let eip712_hash = keccak256([&[0x19, 0x01], &domain_separator[..], &message_hash[..]].concat());

//     Ok(eip712_hash)
// }
