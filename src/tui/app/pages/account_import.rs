use std::sync::{atomic::AtomicBool, mpsc, Arc};

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect},
    widgets::Widget,
};

use crate::{
    tui::{
        app::{widgets::input_box::InputBox, SharedState},
        events::Event,
        traits::{Component, HandleResult},
    },
    utils::account::AccountManager,
};

#[derive(Default)]
pub struct AccountImportPage {
    pub screen: usize,
    pub input: String,
    pub text_cursor: usize,
    pub display: Option<String>,
    pub success: bool,
}

impl Component for AccountImportPage {
    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        let mut result = HandleResult::default();

        if let Event::Input(key_event) = event {
            if self.display.is_some() {
                if self.success {
                    result.page_pops = 1;
                    result.reload = true;
                } else {
                    self.display = None;
                }
                return Ok(result);
            }

            match key_event.code {
                KeyCode::Char(char) => {
                    self.input.push(char);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter => {
                    let import_result = AccountManager::import_mnemonic_wallet(&self.input)
                        .or_else(|_| AccountManager::import_private_key(&self.input, None));

                    match import_result {
                        Ok(address) => {
                            self.display = Some(format!("Successfully imported wallet: {address}"));
                            self.success = true;
                        }
                        Err(err) => {
                            self.display = Some(format!("Error importing wallet: {err}"));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
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
            display.render(area.offset(Offset { x: 0, y: 4 }), buf);
        }

        area
    }
}
