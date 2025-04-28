use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::Widget;
use std::sync::mpsc;
use std::sync::{atomic::AtomicBool, Arc};

use crate::tui::{
    app::widgets::form::{Form, FormItem}, // <- Using your custom form system
    events::Event,
    traits::{Component, HandleResult},
};
use crate::Result;

pub struct SendMessagePage {
    pub to: String,
    pub message: String,
    pub cursor: usize,
    pub error: Option<String>,
    pub status: Option<String>,
}

impl Default for SendMessagePage {
    fn default() -> Self {
        Self {
            to: String::new(),
            message: String::new(),
            cursor: 0,
            error: None,
            status: None,
        }
    }
}

impl Component for SendMessagePage {
    fn handle_event(
        &mut self,
        event: &Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
    ) -> Result<HandleResult> {
        let result = HandleResult::default();

        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Tab | KeyCode::Down => {
                        self.cursor = (self.cursor + 1) % 4;
                    }
                    KeyCode::Up => {
                        self.cursor = (self.cursor + 3) % 4;
                    }
                    KeyCode::Enter => match self.cursor {
                        2 => {
                            self.status = Some("Opened Address Book".into());
                            self.error = None;
                        }
                        3 => {
                            if self.to.trim().is_empty() || self.message.trim().is_empty() {
                                self.error = Some("Recipient and message cannot be empty.".into());
                                self.status = None;
                            } else {
                                self.status = Some("Message sent!".into());
                                self.error = None;
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Char(c) => match self.cursor {
                        0 => self.to.push(c),
                        1 => self.message.push(c),
                        _ => {}
                    },
                    KeyCode::Backspace => match self.cursor {
                        0 => {
                            self.to.pop();
                        }
                        1 => {
                            self.message.pop();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        Ok(result)
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        Form {
            items: vec![
                FormItem::Heading("Send a Message"),
                FormItem::InputBox {
                    focus: self.cursor == 0,
                    label: &"To".to_string(),
                    text: &self.to,
                },
                FormItem::InputBox {
                    focus: self.cursor == 1,
                    label: &"Message".to_string(),
                    text: &self.message,
                },
                FormItem::Button {
                    focus: self.cursor == 2,
                    label: &"Select From Address Book".to_string(),
                },
                FormItem::Button {
                    focus: self.cursor == 3,
                    label: &"Send Message".to_string(),
                },
            ],
        }
        .render(area, buf);

        area
    }
}
