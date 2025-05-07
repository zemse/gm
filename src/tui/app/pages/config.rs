use std::sync::{atomic::AtomicBool, mpsc, Arc};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    disk::{Config, DiskInterface},
    tui::{
        app::{
            widgets::{
                form::{Form, FormItem},
                input_box::InputBox,
            },
            SharedState,
        },
        events::Event,
        traits::{Component, HandleResult},
    },
};

pub struct ConfigPage {
    pub cursor: usize,
    pub config: Config,
    pub display: Option<String>,
}

impl Default for ConfigPage {
    fn default() -> Self {
        let mut config = Config::load();
        if config.alchemy_api_key.is_none() {
            config.alchemy_api_key = Some("".to_string());
        }
        Self {
            cursor: 0,
            config,
            display: None,
        }
    }
}
impl Component for ConfigPage {
    fn text_input_mut(&mut self) -> Option<&mut String> {
        match self.cursor {
            0 => self.config.alchemy_api_key.as_mut(),
            _ => None,
        }
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
    ) -> crate::Result<HandleResult> {
        InputBox::handle_events(self.text_input_mut(), event)?;

        let cursor_max = 2;
        let mut handle_result = HandleResult::default();
        if let Event::Input(key_event) = event {
            self.display = None;

            if key_event.kind == KeyEventKind::Press {
                if self.cursor == 1 {
                    match key_event.code {
                        KeyCode::Char(_)
                        | KeyCode::Left
                        | KeyCode::Right
                        | KeyCode::Tab
                        | KeyCode::Backspace => {
                            self.config.testnet_mode = !self.config.testnet_mode;
                        }
                        _ => {}
                    }
                }

                match key_event.code {
                    KeyCode::Up => {
                        self.cursor = (self.cursor + cursor_max - 1) % cursor_max;
                    }
                    KeyCode::Down => {
                        self.cursor = (self.cursor + 1) % cursor_max;
                    }
                    KeyCode::Enter => {
                        self.config.save();
                        self.display = Some("Configuration saved".to_string());
                        handle_result.reload = true;
                    }
                    KeyCode::Tab => self.cursor += 1,
                    _ => {}
                }
            }
        }

        Ok(handle_result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, _: &SharedState) -> Rect
    where
        Self: Sized,
    {
        Form {
            items: vec![
                FormItem::Heading("Configuration"),
                FormItem::InputBox {
                    focus: self.cursor == 0,
                    label: &"Alchemy API key".to_string(),
                    text: self.config.alchemy_api_key.as_ref().unwrap(),
                },
                FormItem::BooleanInput {
                    focus: self.cursor == 1,
                    label: &"Testnet Mode".to_string(),
                    value: &self.config.testnet_mode,
                },
                FormItem::Text(self.display.as_ref()),
            ],
        }
        .render(area, buf);
        // self.display.render(area, buf);
        area
    }
}
