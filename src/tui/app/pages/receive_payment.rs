// File: src/tui/app/pages/receive_payment.rs

use std::{
    future::Future,
    sync::{Arc, atomic::AtomicBool, mpsc},
    time::{Duration, Instant},
};

use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Widget,
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    actions::receive_payment::PaymentRequest,
    disk::{Config, DiskInterface},
    error::Error,
    tui::{app::SharedState, events::Event, traits::{Component, HandleResult}},
};

/// A TUI page for entering amount & network, then rendering & copying a receive link.
pub struct ReceivePaymentPage {
    amount_input: String,
    networks: Vec<String>,
    selected: usize,
    banner: Option<(String, Instant)>,
}

impl Default for ReceivePaymentPage {
    fn default() -> Self {
        ReceivePaymentPage {
            amount_input: String::new(),
            networks: vec![
                "ethereum".into(),
                "arbitrum".into(),
                "optimism".into(),
                "polygon".into(),
            ],
            selected: 1,
            banner: None,
        }
    }
}

impl Component for ReceivePaymentPage {
    fn reload(&mut self) {
        self.amount_input.clear();
        self.selected = 1;
        self.banner = None;
    }

    fn exit_threads(&mut self) -> impl Future<Output = ()> {
        async {}
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _tx: &mpsc::Sender<Event>,
        _shutdown: &Arc<AtomicBool>,
    ) -> Result<HandleResult, Error> {
        let mut result = HandleResult::default();

        if let Event::Input(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(c) => self.amount_input.push(c),
                    KeyCode::Backspace => { self.amount_input.pop(); }
                    KeyCode::Tab => { self.selected = (self.selected + 1) % self.networks.len(); }
                    KeyCode::Esc => { result.page_pops = 1; }
                    KeyCode::Enter => {
                        // Build request and copy link
                        let req = PaymentRequest {
                            account: Config::load()
                                .current_account
                                .map(|a| a.to_string())
                                .unwrap_or("unknown.eth".to_string()),
                            amount: self.amount_input.clone(),
                            network: self.networks[self.selected].clone(),
                        };
                        let link = req.generate_link();

                        // Copy to clipboard
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        let _ = ctx.set_contents(link.clone());

                        // Show confirmation banner
                        self.banner = Some((format!("Copied link: {}", link), Instant::now()));
                    }
                    _ => {}
                }
            }
        }

        Ok(result)
    }

    fn render_component(
        &self,
        area: Rect,
        buf: &mut Buffer,
        _shared_state: &SharedState,
    ) -> Rect {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(if self.banner.is_some() { 2 } else { 0 }),
            ])
            .split(area);

        // 1) Amount input
        Paragraph::new(self.amount_input.as_str())
            .block(Block::default().title("Amount (e.g. 100USDC)").borders(Borders::ALL))
            .render(chunks[0], buf);

        // 2) Network selector
        let net = &self.networks[self.selected];
        Paragraph::new(net.as_str())
            .block(Block::default().title("Network (Tab)").borders(Borders::ALL))
            .render(chunks[1], buf);

        // 3) Current account display
        let account = Config::load()
            .current_account
            .map(|a| a.to_string())
            .unwrap_or("<none>".to_string());
        Paragraph::new(account.as_str())
            .block(Block::default().title("Account").borders(Borders::ALL))
            .render(chunks[2], buf);

        // 4) Link preview
        let preview = if !self.amount_input.is_empty() {
            let req = PaymentRequest {
                account: Config::load()
                    .current_account
                    .map(|a| a.to_string())
                    .unwrap_or("unknown.eth".to_string()),
                amount: self.amount_input.clone(),
                network: net.clone(),
            };
            req.generate_link()
        } else {
            "<enter amount and press Enter>".into()
        };
        Paragraph::new(preview.as_str())
            .block(Block::default().title("Receive Link (Enter)").borders(Borders::ALL))
            .render(chunks[3], buf);

        // 5) Confirmation banner (2s)
        if let Some((msg, when)) = &self.banner {
            if when.elapsed() < Duration::from_secs(2) {
                Paragraph::new(msg.as_str())
                    .block(Block::default().borders(Borders::ALL))
                    .render(chunks[4], buf);
            }
        }

        area
    }
}