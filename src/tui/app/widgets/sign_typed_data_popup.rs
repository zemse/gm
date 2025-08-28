use std::sync::mpsc;

use alloy::{
    primitives::{keccak256, Address, B256},
    signers::{Signature, Signer},
};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
};
use serde_json::Value;
use tokio::task::JoinHandle;

use crate::{
    error::FmtError,
    tui::{
        app::{
            widgets::{button::Button, popup::Popup, text_scroll::TextScroll},
            SharedState,
        },
        theme::Theme,
        traits::{CustomRender, HandleResult, RectUtil},
        Event,
    },
    utils::{account::AccountManager, eip712::eip712_to_dyn},
};

fn spawn_sign_thread(
    digest: B256,
    tr: &mpsc::Sender<Event>,
    shared_state: &SharedState,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let sender_account = shared_state
        .current_account
        .ok_or(crate::Error::CurrentAccountNotSet)?;

    Ok(tokio::spawn(async move {
        let _ = match run(digest, sender_account).await {
            Ok(sig) => tr.send(Event::SignResult(sig)),
            Err(err) => tr.send(Event::SignError(err.fmt_err("SignError"))),
        };

        async fn run(digest: B256, sender_account: Address) -> crate::Result<Signature> {
            let wallet = AccountManager::load_wallet(&sender_account)?;
            Ok(wallet.sign_hash(&digest).await?)
        }
    }))
}

#[derive(Default)]
enum SignStatus {
    #[default]
    Idle,
    Signing,
    Done,
    Failed,
}

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
            return Err(crate::Error::InternalErrorStr("typed-data missing `types`"));
        }
        if v.get("domain").is_none() {
            return Err(crate::Error::InternalErrorStr(
                "typed-data missing `domain`",
            ));
        }
        if v.get("primaryType").is_none() {
            return Err(crate::Error::InternalErrorStr(
                "typed-data missing `primaryType`",
            ));
        }
        if v.get("message").is_none() {
            return Err(crate::Error::InternalErrorStr(
                "typed-data missing `message`",
            ));
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
        (event, area, tr, ss): (&crate::tui::Event, Rect, &mpsc::Sender<Event>, &SharedState),
        mut on_signature: F1,
        mut on_cancel: F3,
        mut on_esc: F4,
    ) -> crate::Result<HandleResult>
    where
        F1: FnMut(&Signature) -> crate::Result<()>,
        F3: FnMut() -> crate::Result<()>,
        F4: FnMut() -> crate::Result<()>,
    {
        let mut result = HandleResult::default();

        if self.is_open() {
            let area = Popup::inner_area(area).block_inner().margin_down(3);

            let r = self.display.handle_event(event, area)?;
            result.merge(r);

            match event {
                Event::Input(key_event) if key_event.kind == KeyEventKind::Press => {
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
                                    let digest = eip712_digest_from_json(&self.typed_data_json)?;
                                    self.status = SignStatus::Signing;
                                    self.sign_thread = Some(spawn_sign_thread(digest, tr, ss)?);
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
                Event::SignResult(signature) => {
                    on_signature(signature)?;
                    self.status = SignStatus::Done;

                    if let Some(thread) = self.sign_thread.take() {
                        thread.abort();
                    }
                }
                Event::SignError(_) => {
                    self.status = SignStatus::Failed;
                }
                _ => {}
            }
            result.esc_ignores = 1;
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

            self.display.render(text_area, buf);

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

fn eip712_digest_from_json(typed_data: &Value) -> crate::Result<B256> {
    let (_msg_type, message_value, _domain_type, domain_value) = eip712_to_dyn(typed_data)?;

    let encoded_domain = domain_value.abi_encode();
    let encoded_message = message_value.abi_encode();

    let domain_separator = keccak256(&encoded_domain);
    let message_hash = keccak256(&encoded_message);
    let eip712_hash = keccak256([&[0x19, 0x01], &domain_separator[..], &message_hash[..]].concat());

    Ok(eip712_hash)
}
