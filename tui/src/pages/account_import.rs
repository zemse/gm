use std::{borrow::Cow, sync::mpsc};

use gm_ratatui_extra::input_box::InputBox;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::{Offset, Rect},
    widgets::WidgetRef,
};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState, post_handle_event::PostHandleEventActions, traits::Component, AppEvent,
};
use gm_utils::account::AccountManager;

#[derive(Debug)]
pub struct AccountImportPage {
    pub screen: usize,
    pub input_box: InputBox,
    pub display: Option<String>,
    pub success: bool,
}

impl Default for AccountImportPage {
    fn default() -> Self {
        Self {
            screen: 0,
            input_box: InputBox::new("Private key or Mnemonic phrase"),
            display: None,
            success: false,
        }
    }
}

impl Component for AccountImportPage {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Import")
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _popup_area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let mut actions = PostHandleEventActions::default();

        self.input_box
            .handle_event(event.widget_event().as_ref(), area, &mut actions);

        if let AppEvent::Input(input_event) = event {
            if self.display.is_some() {
                if self.success {
                    actions.page_pop();
                    actions.reload();
                } else {
                    self.display = None;
                }
                return Ok(actions);
            }

            match input_event {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Enter {
                        let import_result =
                            AccountManager::import_mnemonic_wallet(self.input_box.get_text())
                                .or_else(|_| {
                                    AccountManager::import_private_key(self.input_box.get_text())
                                });

                        match import_result {
                            Ok(address) => {
                                self.display =
                                    Some(format!("Successfully imported wallet: {address}"));
                                self.success = true;
                            }
                            Err(err) => {
                                self.display = Some(format!("Error importing wallet: {err}"));
                            }
                        }
                    }
                }
                Event::Mouse(_) => {}
                _ => {}
            }
        }

        Ok(actions)
    }

    fn render_component(
        &self,
        area: Rect,
        _popup_area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        self.input_box.render(area, buf, true, &shared_state.theme);

        if let Some(display) = &self.display {
            display.render_ref(area.offset(Offset { x: 0, y: 4 }), buf);
        }

        area
    }
}
