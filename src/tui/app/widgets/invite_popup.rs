use std::{sync::mpsc, time::Duration};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Widget},
};
use tokio::task::JoinHandle;

use crate::tui::{
    app::{widgets::popup::Popup, SharedState},
    traits::{CustomRender, HandleResult},
    Event,
};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum InviteCodeValidity {
    #[default]
    Checking,
    Valid,
    Invalid,
    Claimed,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum InviteCodeClaimStatus {
    #[default]
    Idle,
    Claiming,
    Success,
    Failed(String),
}

pub fn start_check_thread(
    invite_code: &str,
    tr: &mpsc::Sender<Event>,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let invite_code = invite_code.to_string();
    Ok(tokio::spawn(async move {
        // TODO make api call to backend to check invite code validity
        let _ = invite_code;
        let result = InviteCodeValidity::Valid;
        tokio::time::sleep(Duration::from_secs(1)).await;

        let _ = tr.send(Event::InviteCodeValidity(result));
    }))
}

pub fn start_claim_thread(
    invite_code: &str,
    tr: &mpsc::Sender<Event>,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let invite_code = invite_code.to_string();
    Ok(tokio::spawn(async move {
        let _ = tr.send(Event::InviteCodeClaimStatus(
            InviteCodeClaimStatus::Claiming,
        ));
        // TODO make api call to backend to claim invite code
        let _ = invite_code;
        tokio::time::sleep(Duration::from_secs(1)).await;

        let _ = tr.send(Event::InviteCodeClaimStatus(InviteCodeClaimStatus::Success));
    }))
}

#[derive(Default)]
pub struct InvitePopup {
    invite_code: Option<String>,
    validity: InviteCodeValidity,
    claim_status: InviteCodeClaimStatus,
    check_thread: Option<JoinHandle<()>>,
    claim_thread: Option<JoinHandle<()>>,
    open: bool,
}

impl InvitePopup {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn set_invite_code(&mut self, text: &str) {
        self.reset();
        self.invite_code = Some(text.to_string());
    }

    fn reset(&mut self) {
        if let Some(thread) = self.check_thread.take() {
            thread.abort();
        }

        if let Some(thread) = self.claim_thread.take() {
            thread.abort();
        }
    }

    pub fn handle_event(
        &mut self,
        event: &Event,
        tr: &mpsc::Sender<Event>,
    ) -> crate::Result<HandleResult> {
        let mut result = HandleResult::default();

        if self.check_thread.is_none() {
            if let Some(invite_code) = self.invite_code.as_ref() {
                let check_thread = start_check_thread(invite_code, tr)?;
                self.check_thread = Some(check_thread);
            }
        }

        match event {
            Event::Input(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Enter => {
                            if self.validity == InviteCodeValidity::Valid
                                && self.claim_status == InviteCodeClaimStatus::Idle
                            {
                                if let Some(invite_code) = self.invite_code.as_ref() {
                                    let claim_thread = start_claim_thread(invite_code, tr)?;
                                    self.claim_thread = Some(claim_thread);
                                }
                            }
                        }
                        KeyCode::Esc => {
                            self.close();
                            result.esc_ignores = 1;
                        }
                        _ => {}
                    }
                }
            }
            Event::InviteCodeValidity(validity) => {
                self.validity = *validity;
            }
            Event::InviteCodeClaimStatus(status) => {
                self.claim_status = status.clone();
            }
            _ => {}
        }
        result.esc_ignores = 1;
        Ok(result)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState)
    where
        Self: Sized,
    {
        if self.is_open() {
            // if wallet address exists
            // we will start invite process
            // if wallet address does not exist
            // we will suggest user to create a new account, even show a button - "go to create new account"
            let theme = shared_state.theme.popup();

            Popup.render(area, buf, &theme);

            let inner_area = Popup::inner_area(area);
            let block = Block::bordered();
            let block_inner_area = block.inner(inner_area);
            block.render(inner_area, buf);

            let area = block_inner_area;

            [
                "Welcome! And thanks for joining gm's alpha testing program!".to_string(),
                if let Some(invite_code) = self.invite_code.as_ref() {
                    match self.validity {
                        InviteCodeValidity::Checking => {
                            format!("Invite Code: \"{invite_code}\", checking validity...")
                        }
                        InviteCodeValidity::Valid => {
                            format!("Invite code: \"{invite_code}\", valid!")
                        }
                        InviteCodeValidity::Invalid => {
                            format!(
                                "Invite Code: \"{invite_code}\" is invalid, please check the code"
                            )
                        }
                        InviteCodeValidity::Claimed => {
                            format!("Invite Code: \"{invite_code}\", claimed already")
                        }
                    }
                } else {
                    "Invite Code not provided, this should not happen".to_string()
                },
                match self.claim_status {
                    InviteCodeClaimStatus::Idle => {
                        if self.validity == InviteCodeValidity::Valid {
                            "Press Enter to claim".to_string()
                        } else {
                            "".to_string()
                        }
                    }
                    InviteCodeClaimStatus::Claiming => "Claiming invite code...".to_string(),
                    InviteCodeClaimStatus::Success => "Claimed successfully!".to_string(),
                    InviteCodeClaimStatus::Failed(ref msg) => {
                        format!("Failed to claim invite code: {msg}")
                    }
                },
            ]
            .render(area, buf, true);
        }
    }
}
