use std::sync::mpsc;

use alloy::primitives::Address;

use gm_ratatui_extra::{
    act::Act,
    extensions::{RenderTextWrapped, ThemedWidget},
    popup::{Popup, PopupWidget},
    thematize::Thematize,
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::Rect,
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
    popup: Popup,
    invite_code: Option<String>,
    validity: InviteCodeValidity,
    claim_status: InviteCodeClaimStatus,
    check_thread: Option<JoinHandle<()>>,
    claim_thread: Option<JoinHandle<()>>,
}

impl PopupWidget for InvitePopup {
    fn get_popup(&self) -> &Popup {
        &self.popup
    }

    fn get_popup_mut(&mut self) -> &mut Popup {
        &mut self.popup
    }
}

impl InvitePopup {
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
        actions: &mut PostHandleEventActions,
    ) -> crate::Result<()> {
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
                                actions.ignore_esc();
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
        actions.ignore_esc();

        Ok(())
    }

    pub fn render(&self, popup_area: Rect, buf: &mut Buffer, shared_state: &SharedState)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = shared_state.theme.popup();
            self.popup.render(popup_area, buf, &theme);

            vec![
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
            .render_wrapped(self.body_area(popup_area), buf);
        }
    }
}
