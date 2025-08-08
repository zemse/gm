use std::sync::mpsc;

use alloy::{
    hex,
    primitives::Address,
    signers::{Signature, Signer},
};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
};
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
    utils::account::AccountManager,
};

pub fn sign_thread(
    message: &str,
    tr: &mpsc::Sender<Event>,
    shared_state: &SharedState,
) -> crate::Result<JoinHandle<()>> {
    let message = message.to_string();
    let tr = tr.clone();
    let sender_account = shared_state.try_current_account()?;

    Ok(tokio::spawn(async move {
        let _ = match run(message, sender_account).await {
            Ok(sig) => tr.send(Event::SignResult(sig)),
            Err(err) => tr.send(Event::SignError(err.fmt_err("SignError"))),
        };

        async fn run(message: String, sender_account: Address) -> crate::Result<Signature> {
            let wallet = AccountManager::load_wallet(&sender_account)?;

            let data = match hex::decode(&message) {
                Ok(bytes) => bytes,
                Err(_) => message.as_bytes().to_vec(),
            };
            Ok(wallet.sign_message(&data).await?)
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

#[derive(Default)]
pub struct SignPopup {
    text: TextScroll,
    open: bool,
    button_cursor: bool, // is cursor on the confirm button?
    status: SignStatus,
    sign_thread: Option<JoinHandle<()>>,
}

impl SignPopup {
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

    pub fn set_text(&mut self, text: &str) {
        self.text.text = text.to_string();
        self.reset();
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

        let r = self.text.handle_event(event, area)?;
        result.merge(r);

        match event {
            Event::Input(key_event) => {
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
                                    self.status = SignStatus::Signing;
                                    self.sign_thread = Some(sign_thread(&self.text.text, tr, ss)?);
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
            let block = Block::bordered().title("Sign Message");
            let block_inner_area = block.inner(inner_area);
            block.render(inner_area, buf);

            let [text_area, button_area] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(3)])
                    .areas(block_inner_area);

            self.text.render(text_area, buf);

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
                    "Signing message...".render(button_area.margin_top(1), buf);
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
