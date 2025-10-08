use std::sync::mpsc;

use alloy::primitives::Address;

use gm_ratatui_extra::{act::Act, extensions::CustomRender, popup::Popup, thematize::Thematize};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::Rect,
    widgets::{Block, Widget},
};
use serde_json::json;
use tokio::task::JoinHandle;

use crate::{
    app::SharedState, error::FmtError, post_handle_event::PostHandleEventActions, AppEvent,
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

const BASE_URL: &str = "https://invites.gm-tui.com";

pub fn start_check_thread(
    invite_code: &str,
    tr: &mpsc::Sender<AppEvent>,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let invite_code = invite_code.to_string();
    Ok(tokio::spawn(async move {
        let res: Result<(), crate::Error> = async {
            let result = gm_utils::Reqwest::get(format!("{BASE_URL}/check"))?
                .query(&json!({"invite_code": invite_code}))
                .receive_text()
                .await?;

            let validity = match result.as_str() {
                "claimed" => InviteCodeValidity::Claimed,
                "valid" => InviteCodeValidity::Valid,
                _ => InviteCodeValidity::Invalid,
            };

            let _ = tr.send(AppEvent::InviteCodeValidity(validity));
            Ok(())
        }
        .await;

        if let Err(e) = res {
            let _ = tr.send(AppEvent::InviteError(e.fmt_err("InviteCheckError")));
        }
    }))
}

pub fn start_claim_thread(
    invite_code: &str,
    claim_address: Address,
    tr: &mpsc::Sender<AppEvent>,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let invite_code = invite_code.to_string();
    Ok(tokio::spawn(async move {
        let res: crate::Result<()> = async {
            let _ = tr.send(AppEvent::InviteCodeClaimStatus(
                InviteCodeClaimStatus::Claiming,
            ));

            let out = gm_utils::Reqwest::post(format!("{BASE_URL}/claim"))?
                .json_body(&json!({"invite_code": invite_code, "address": claim_address}))
                .receive_text()
                .await?;

            if out.len() > 10 {
                let _ = tr.send(AppEvent::InviteCodeClaimStatus(
                    InviteCodeClaimStatus::Success,
                ));
                let _ = tr.send(AppEvent::InviteCodeClaimStatus(
                    InviteCodeClaimStatus::Failed("failed".to_string()),
                ));
            }
            Ok(())
        }
        .await;

        if let Err(e) = res {
            let _ = tr.send(AppEvent::InviteError(e.fmt_err("InviteCheckError")));
        }
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

    pub fn set_invite_code(&mut self, text: String) {
        self.reset();
        self.invite_code = Some(text);
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
        event: &AppEvent,
        tr: &mpsc::Sender<AppEvent>,
        ss: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let mut result = PostHandleEventActions::default();

        if self.check_thread.is_none() {
            if let Some(invite_code) = self.invite_code.as_ref() {
                let check_thread = start_check_thread(invite_code, tr)?;
                self.check_thread = Some(check_thread);
            }
        }

        match event {
            AppEvent::Input(input_event) => match input_event {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Enter => {
                                if self.validity == InviteCodeValidity::Valid
                                    && self.claim_status == InviteCodeClaimStatus::Idle
                                {
                                    if let Some(invite_code) = self.invite_code.as_ref() {
                                        let claim_thread = start_claim_thread(
                                            invite_code,
                                            ss.try_current_account()?,
                                            tr,
                                        )?;
                                        self.claim_thread = Some(claim_thread);
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                self.close();
                                result.ignore_esc();
                            }
                            _ => {}
                        }
                    }
                }
                Event::Mouse(_mouse_event) => {}
                _ => {}
            },
            AppEvent::InviteCodeValidity(validity) => {
                self.validity = *validity;
            }
            AppEvent::InviteCodeClaimStatus(status) => {
                self.claim_status = status.clone();
            }
            _ => {}
        }
        result.ignore_esc();
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
