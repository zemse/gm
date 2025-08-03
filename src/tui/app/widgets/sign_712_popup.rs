use std::{fmt::Debug, sync::mpsc};

use alloy::{
    dyn_abi::Eip712Domain,
    primitives::{Address, B256},
    signers::{Signature, Signer},
    sol_types::SolStruct,
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
        let _ = match run(digest, sender_account.as_raw()).await {
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

pub struct Sign712Popup<S: SolStruct + Debug + Default> {
    domain: Eip712Domain,
    data: S,
    display: TextScroll,
    open: bool,
    button_cursor: bool, // is cursor on the confirm button?
    status: SignStatus,
    sign_thread: Option<JoinHandle<()>>,
}

impl<S: SolStruct + Debug + Default> Sign712Popup<S> {
    pub fn new(domain: Eip712Domain) -> Self {
        Self {
            domain,
            data: S::default(),
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

    pub fn set_data_struct(&mut self, data: S) {
        self.display.text = format!("Domain: {:#?}\nData: {:#?}\n\n\n\n\n\n", self.domain, data);
        self.data = data;
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
        F1: FnMut(&Signature, &mut Self) -> crate::Result<()>,
        F3: FnMut() -> crate::Result<()>,
        F4: FnMut() -> crate::Result<()>,
    {
        let mut result = HandleResult::default();

        if self.is_open() {
            // Text area is popup block inner and subtracting 3 lines of button area
            let area = Popup::inner_area(area).block_inner().margin_down(3);

            let r = self.display.handle_event(event, area)?;
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
                                        self.sign_thread = Some(spawn_sign_thread(
                                            self.data.eip712_signing_hash(&self.domain),
                                            tr,
                                            ss,
                                        )?);
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
                    on_signature(signature, self)?;
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
            let block = Block::bordered().title("Sign EIP712 Struct");
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
