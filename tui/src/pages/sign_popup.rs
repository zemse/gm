use std::sync::mpsc;

use alloy::{
    hex,
    primitives::Address,
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
use gm_utils::account::AccountManager;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
};
use tokio::task::JoinHandle;

use crate::{app::SharedState, theme::Theme, traits::Actions, AppEvent};

pub fn sign_thread(
    message: &str,
    tr: &mpsc::Sender<AppEvent>,
    shared_state: &SharedState,
) -> crate::Result<JoinHandle<()>> {
    let message = message.to_string();
    let tr = tr.clone();
    let sender_account = shared_state.try_current_account()?;

    Ok(tokio::spawn(async move {
        let _ = match run(message, sender_account).await {
            Ok(sig) => tr.send(AppEvent::SignResult(sig)),
            // TODO have `run` return a scoped error so we don't have to send back string
            Err(err) => tr.send(AppEvent::SignError(format!("{err:?}"))),
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

#[derive(Default, Debug)]
enum SignStatus {
    #[default]
    Idle,
    Signing,
    Done,
    Failed,
}

pub enum SignPopupEvent {
    Signed(Signature),
    Rejected,
    EscapedBeforeSigning,
    EscapedAfterSigning,
}

#[derive(Default, Debug)]
pub struct SignPopup {
    msg_hex: String,
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
        self.button_cursor = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn set_msg_hex(&mut self, msg_hex: &str) {
        self.msg_hex = msg_hex.to_string();
        let utf8_str = hex::decode(msg_hex)
            .map_err(crate::Error::FromHexError)
            .and_then(|bytes| String::from_utf8(bytes).map_err(crate::Error::FromUtf8Error));

        self.text.text = utf8_str.unwrap_or(self.msg_hex.clone());
        self.reset();
    }

    fn reset(&mut self) {
        self.button_cursor = false;
        self.status = SignStatus::Idle;
        if let Some(thread) = self.sign_thread.take() {
            thread.abort();
        }
    }

    pub fn handle_event<F>(
        &mut self,
        (event, area, tr, ss): (&AppEvent, Rect, &mpsc::Sender<AppEvent>, &SharedState),
        mut on_event: F,
    ) -> crate::Result<Actions>
    where
        F: FnMut(SignPopupEvent) -> crate::Result<()>,
    {
        let mut result = Actions::default();

        self.text.handle_event(event.key_event(), area);

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
                                        self.status = SignStatus::Signing;
                                        self.sign_thread =
                                            Some(sign_thread(&self.text.text, tr, ss)?);
                                    } else {
                                        self.close();
                                        on_event(SignPopupEvent::Rejected)?;
                                    }
                                }
                                KeyCode::Esc => {
                                    self.close();
                                    on_event(SignPopupEvent::EscapedBeforeSigning)?;
                                }
                                _ => {}
                            },
                            SignStatus::Signing => {}
                            SignStatus::Done | SignStatus::Failed => {
                                if key_event.code == KeyCode::Esc {
                                    self.close();
                                    on_event(SignPopupEvent::EscapedAfterSigning)?;
                                }
                            }
                        }
                    }
                }
                Event::Mouse(_mouse_event) => {}
                _ => {}
            },
            AppEvent::SignResult(signature) => {
                on_event(SignPopupEvent::Signed(*signature))?;
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
        result.ignore_esc();
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

            self.text.render(text_area, buf, &theme);

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
