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

#[derive(Default, Debug)]
pub struct AccountImportPage {
    pub screen: usize,
    pub input: String,
    pub text_cursor: usize,
    pub display: Option<String>,
    pub success: bool,
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
        let mut result = PostHandleEventActions::default();

        InputBox::handle_event(
            event.input_event(),
            area,
            &mut self.input,
            &mut self.text_cursor,
        );

        if let AppEvent::Input(input_event) = event {
            if self.display.is_some() {
                if self.success {
                    result.page_pop();
                    result.reload();
                } else {
                    self.display = None;
                }
                return Ok(result);
            }

            match input_event {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Enter {
                        let import_result = AccountManager::import_mnemonic_wallet(&self.input)
                            .or_else(|_| AccountManager::import_private_key(&self.input));

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

        Ok(result)
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
        InputBox {
            focus: true,
            label: "Private key or Mnemonic phrase",
            text: &self.input,
            empty_text: None,
            currency: None,
        }
        .render(area, buf, &self.text_cursor, &shared_state.theme);

        if let Some(display) = &self.display {
            display.render_ref(area.offset(Offset { x: 0, y: 4 }), buf);
        }

        area
    }
}
