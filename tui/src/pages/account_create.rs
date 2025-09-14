use std::{
    sync::{atomic::AtomicBool, mpsc, Arc},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use alloy::primitives::{address, Address};
use gm_ratatui_extra::thematize::Thematize;
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Offset, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Gauge, Widget},
};

use crate::{
    app::SharedState,
    events::Event,
    traits::{Actions, Component},
};
use gm_utils::account::{mine_wallet, AccountManager, AccountUtils};

#[derive(PartialEq)]
pub enum HashRateResult {
    None,
    Pending,
    Some(usize),
    Error(String),
}

pub struct AccountCreatePage {
    pub cursor: usize,
    pub mask: [Option<u8>; 40],
    pub error: Option<String>,
    pub hash_rate_thread: Option<JoinHandle<()>>,
    pub hash_rate: HashRateResult,
    pub mining: bool,
    pub started_mining_at: Instant,
    pub vanity_thread: Option<JoinHandle<()>>,
    pub vanity_result: Option<(Address, usize, Duration)>,
    pub mnemonic_result: Option<Address>,
}

impl Default for AccountCreatePage {
    fn default() -> Self {
        Self {
            cursor: 0,
            mask: [None; 40],
            error: None,
            hash_rate_thread: None,
            hash_rate: HashRateResult::None,
            mining: false,
            started_mining_at: Instant::now(),
            vanity_thread: None,
            vanity_result: None,
            mnemonic_result: None,
        }
    }
}

impl AccountCreatePage {
    pub fn is_mask_empty(&self) -> bool {
        self.mask.iter().all(|&x| x.is_none())
    }

    pub fn mask_count(&self) -> usize {
        self.mask.iter().filter(|&&x| x.is_some()).count()
    }

    pub fn mask_a_b(&self) -> (Address, Address) {
        let mut mask_a = [0; 20];
        let mut mask_b = [0; 20];

        for (i, &b) in self.mask.iter().enumerate() {
            if let Some(n) = b {
                mask_a[i / 2] |= n << ((1 - i % 2) * 4);
                mask_b[i / 2] |= (0xf ^ n) << ((1 - i % 2) * 4);
            }
        }

        (Address::from(mask_a), Address::from(mask_b))
    }
}

impl Component for AccountCreatePage {
    async fn exit_threads(&mut self) {
        if let Some(thread) = self.hash_rate_thread.take() {
            thread.join().unwrap();
        }
        if let Some(thread) = self.vanity_thread.take() {
            thread.join().unwrap();
        }
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        transmitter: &mpsc::Sender<Event>,
        shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let result = Actions::default();

        let cursor_max = self.mask.len();
        match event {
            Event::Input(key_event) => match key_event.code {
                KeyCode::Right => {
                    self.cursor = (self.cursor + 1) % cursor_max;
                }
                KeyCode::Left => {
                    self.cursor = (self.cursor + cursor_max - 1) % cursor_max;
                }
                KeyCode::Char(c) => match c {
                    '0'..='9' => {
                        self.mask[self.cursor] = Some(c as u8 - b'0');
                        if self.cursor < cursor_max - 1 {
                            self.cursor += 1;
                        }
                    }
                    'a'..='f' => {
                        self.mask[self.cursor] = Some(c as u8 - b'a' + 10);
                        if self.cursor < cursor_max - 1 {
                            self.cursor += 1;
                        }
                    }
                    'A'..='F' => {
                        self.mask[self.cursor] = Some(c as u8 - b'A' + 10);
                        if self.cursor < cursor_max - 1 {
                            self.cursor += 1;
                        }
                    }
                    _ => {}
                },
                KeyCode::Backspace => {
                    if self.cursor > 0 {
                        self.mask[self.cursor - 1] = None;
                        self.cursor -= 1;
                    }
                    if self.cursor == 0 || self.cursor == cursor_max {
                        self.mask[self.cursor] = None;
                    }
                }
                KeyCode::Enter => {
                    if self.is_mask_empty() {
                        let addr = AccountManager::create_mnemonic_wallet()?;
                        self.mnemonic_result = Some(addr);
                    }

                    if !self.mining {
                        self.mining = true;
                        self.started_mining_at = Instant::now();
                        let tr = transmitter.clone();
                        let (mask_a, mask_b) = self.mask_a_b();
                        let shutdown_signal = shutdown_signal.clone();
                        let vanity_thread = thread::spawn(move || {
                            let result = mine_wallet(mask_a, mask_b, None, shutdown_signal);
                            if let Ok((Some(key), counter, duration)) = result {
                                tr.send(Event::VanityResult(key, counter, duration))
                                    .unwrap();
                            }
                        });
                        self.vanity_thread = Some(vanity_thread);
                    }
                }
                _ => {}
            },
            Event::HashRateResult(hash_rate) => {
                self.hash_rate = HashRateResult::Some(*hash_rate as usize);
            }
            Event::HashRateError(error) => {
                self.hash_rate = HashRateResult::Error(error.clone());
            }
            Event::VanityResult(key, counter, duration) => {
                let addr = Address::from_private_key(key);
                AccountManager::store_private_key(&key.to_bytes(), addr)?;
                self.vanity_result = Some((addr, *counter, *duration));
                self.hash_rate = HashRateResult::None;
                self.mining = false;
            }
            _ => {}
        }

        if self.hash_rate == HashRateResult::None {
            self.hash_rate = HashRateResult::Pending;

            let tr = transmitter.clone();
            let shutdown_signal = shutdown_signal.clone();
            let hash_rate_thread = thread::spawn(move || {
                let address_one = address!("0xffffffffffffffffffffffffffffffffffffffff");
                let result = mine_wallet(
                    Address::ZERO,
                    address_one,
                    Some(Duration::from_secs(1)),
                    shutdown_signal,
                );
                match result {
                    Ok((_, counter, duration)) => {
                        let hash_rate = counter as f64 / duration.as_secs_f64();
                        tr.send(Event::HashRateResult(hash_rate)).unwrap();
                    }
                    Err(e) => {
                        tr.send(Event::HashRateError(e.to_string())).unwrap();
                    }
                }
            });
            self.hash_rate_thread = Some(hash_rate_thread);
        }

        Ok(result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        Line::from("Create Wallet").bold().render(area, buf);

        "You can edit mask if you wish to vanity generate special address"
            .render(area.offset(Offset { x: 0, y: 3 }), buf);

        "0x".render(area.offset(Offset { x: 0, y: 5 }), buf);

        for (i, b) in self.mask.iter().enumerate() {
            let content = if let Some(n) = b {
                match n {
                    0..=9 => (b'0' + n) as char,
                    10..=15 => (b'a' + (n - 10)) as char,
                    _ => unreachable!("Only 0..=15 allowed"),
                }
            } else {
                '.'
            };
            let span = Span::from(content.to_string());

            let style = if self.cursor == i {
                Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default()
            };

            span.style(style).render(
                area.offset(Offset {
                    x: 2 + i as i32,
                    y: 5,
                }),
                buf,
            );
        }

        let text = if self.is_mask_empty() {
            "Press enter to generate address instantly".to_string()
        } else if let HashRateResult::Some(hash_rate) = self.hash_rate {
            let count = self.mask_count();
            let est_attempts = 16_usize.pow(count as u32);
            let est_time = est_attempts as f64 / hash_rate as f64;
            let est_time = Duration::from_secs(est_time as u64);

            if est_time.as_secs() > 0 {
                format!(
                    "Estimated to take {}, press enter to generate your vanity address",
                    humantime::format_duration(est_time)
                )
            } else {
                "Press enter to generate your vanity address instantly".to_string()
            }
        } else {
            "Press enter to generate address, it may take a while".to_string()
        };

        text.render(area.offset(Offset { x: 0, y: 8 }), buf);

        format!(
            "Hash rate: {}",
            match self.hash_rate {
                HashRateResult::None => "None".to_string(),
                HashRateResult::Pending => "Pending...".to_string(),
                HashRateResult::Some(hash_rate) => format!("{hash_rate} H/s"),
                HashRateResult::Error(ref error) => format!("Error: {error}"),
            }
        )
        .render(area.offset(Offset { x: 0, y: 10 }), buf);

        if self.mining {
            "Mining...".render(area.offset(Offset { x: 0, y: 12 }), buf);
            if let HashRateResult::Some(hash_rate) = self.hash_rate {
                let count = self.mask_count();
                let est_attempts = 16_usize.pow(count as u32);
                let est_time = est_attempts as f64 / hash_rate as f64;
                let elapsed_time = self.started_mining_at.elapsed();
                let remaining_time = est_time - elapsed_time.as_secs_f64();

                if remaining_time.is_sign_negative() {
                    format!(
                        "Searched for: {}.\
                        The expected time has passed, but a match could happen any moment now.",
                        humantime::format_duration(
                            Duration::from_secs(remaining_time.abs() as u64)
                        )
                    )
                    .render(area.offset(Offset { x: 0, y: 16 }), buf);
                } else {
                    format!(
                        "Remaining time: {}",
                        humantime::format_duration(Duration::from_secs(remaining_time as u64))
                    )
                    .render(area.offset(Offset { x: 0, y: 16 }), buf);
                }
                Gauge::default()
                    // TODO rename theme.block to theme.style, or add method
                    .gauge_style(shared_state.theme.block())
                    .percent(
                        (elapsed_time.as_secs() * 100)
                            .checked_div(est_time as u64)
                            .unwrap_or(100) as u16,
                    )
                    .render(Rect::new(area.x, 16, area.width, 1), buf);
            }
        } else if let Some((addr, counter, duration)) = self.vanity_result {
            format!(
                "Vanity mined the address: {}, took {} to perform {} iters",
                addr,
                humantime::format_duration(Duration::from_secs(duration.as_secs())),
                counter
            )
            .render(area.offset(Offset { x: 0, y: 12 }), buf);
        }
        area
    }
}
