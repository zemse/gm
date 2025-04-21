use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use crate::tui::{
    events::Event,
    traits::{Component, HandleResult},
};

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
    fn handle_event(&mut self, event: &Event) -> HandleResult {
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
                    KeyCode::Enter => {
                        match self.cursor {
                            2 => {
                                self.status = Some("📒 Open Address Book".into());
                            }
                            3 => {
                                self.status = Some("Send Message (not ".into());
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char(c) => {
                        match self.cursor {
                            0 => self.to.push(c),
                            1 => self.message.push(c),
                            _ => {}
                        }
                    }
                    KeyCode::Backspace => {
                        match self.cursor {
                            0 => { self.to.pop(); },
                            1 => { self.message.pop(); },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        result
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer) -> Rect {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area);

        let highlight_style = Style::default().fg(Color::Cyan);

        let to_input = Paragraph::new(Text::from(self.to.as_str()))
            .block(Block::default()
                .title("Recipient Address")
                .borders(Borders::ALL)
                .border_style(if self.cursor == 0 { highlight_style } else { Style::default() })
            );
        to_input.render(chunks[0], buf);

        let message_input = Paragraph::new(Text::from(self.message.as_str()))
            .block(Block::default()
                .title("Message")
                .borders(Borders::ALL)
               
                .border_style(if self.cursor == 1 { highlight_style } else { Style::default() })
            );
        message_input.render(chunks[1], buf);

        let address_book_button = Paragraph::new(Text::from("Select from Address Book"))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(if self.cursor == 2 { highlight_style } else { Style::default() })
            );
        address_book_button.render(chunks[2], buf);

        let submit_button = Paragraph::new(Text::from("Send Message"))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(if self.cursor == 3 { highlight_style } else { Style::default() })
            );
        submit_button.render(chunks[3], buf);

        if let Some(err) = &self.error {
            let error_msg = Paragraph::new(Text::from(err.as_str()))
                .style(Style::default().fg(Color::Red));
            error_msg.render(chunks[4], buf);
        }

        if let Some(status) = &self.status {
            let status_msg = Paragraph::new(Text::from(status.as_str()))
                .style(Style::default().fg(Color::Green));
            status_msg.render(chunks[5], buf);
        }

        area
    }
}
