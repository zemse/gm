use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use alloy::primitives::{address, Address};
use gm_ratatui_extra::{act::Act, confirm_popup::ConfirmPopup, thematize::Thematize};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Offset, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Gauge, Widget},
};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState,
    events::Event,
    traits::{Actions, Component},
};
use gm_utils::account::{mine_wallet, AccountManager, AccountUtils};

#[derive(Debug, PartialEq)]
pub enum HashRateResult {
    None,
    Pending,
    Some(usize),
    Error(String),
}

#[derive(Debug)]
pub struct AccountCreatePage {
    cursor: usize,
    mask: [Option<u8>; 40],
    hash_rate: HashRateResult,
    vanity_result: Option<(Address, u64, Duration)>,
    mnemonic_result: Option<Address>,
    mining: bool,
    started_mining_at: Instant,

    exit_signal: CancellationToken,
    hash_rate_thread: Option<JoinHandle<()>>,
    vanity_thread: Option<JoinHandle<()>>,

    exit_popup: ConfirmPopup,
}

impl Default for AccountCreatePage {
    fn default() -> Self {
        Self {
            cursor: 0,
            mask: [None; 40],
            hash_rate: HashRateResult::None,
            vanity_result: None,
            mnemonic_result: None,
            mining: false,
            started_mining_at: Instant::now(),

            exit_signal: CancellationToken::new(),
            hash_rate_thread: None,
            vanity_thread: None,

            exit_popup: ConfirmPopup::new(
                "Warning",
                "Mining will be ended if you go back. You can press ESC to go back or select End. If you want to continue mining you can choose to wait."
                    .to_string(),
                "Wait",
                "Exit",
                false
            ),
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
        self.exit_signal.cancel();

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
        area: Rect,
        transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let mut result = Actions::default();

        if self.exit_popup.is_open() {
            let r = self.exit_popup.handle_event(
                event.key_event(),
                area,
                || Ok(()),
                || -> crate::Result<()> {
                    result.page_pops += 1;
                    result.reload = true;
                    Ok(())
                },
            )?;
            result.merge(r);
        }

        let cursor_max = self.mask.len();
        match event {
            Event::Input(key_event) => {
                if self.mining && key_event.code == KeyCode::Esc {
                    result.ignore_esc();
                    self.exit_popup.open();
                } else {
                    // Handle input only if not mining
                    match key_event.code {
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
                            if self.cursor == 0
                                || (self.cursor == cursor_max - 1
                                    && self.mask[self.cursor].is_some())
                            {
                                self.mask[self.cursor] = None;
                            } else if self.cursor > 0 {
                                self.mask[self.cursor - 1] = None;
                                self.cursor -= 1;
                            }
                        }
                        KeyCode::Enter => {
                            if self.is_mask_empty() {
                                let addr = AccountManager::create_mnemonic_wallet()?;
                                self.mnemonic_result = Some(addr);
                            }

                            if !self.mining && self.vanity_result.is_none() {
                                self.mining = true;
                                self.started_mining_at = Instant::now();
                                let tr = transmitter.clone();
                                let (mask_a, mask_b) = self.mask_a_b();
                                let exit_signal = self.exit_signal.clone();
                                let vanity_thread = thread::spawn(move || {
                                    let result = mine_wallet(mask_a, mask_b, None, exit_signal);
                                    if let Ok((Some(key), counter, duration)) = result {
                                        tr.send(Event::VanityResult(key, counter, duration))
                                            .unwrap();
                                    }
                                });
                                self.vanity_thread = Some(vanity_thread);
                            }
                        }
                        KeyCode::Esc => {
                            // When context goes back to previous page, it should reload state
                            result.reload = true;
                        }
                        _ => {}
                    }
                }
            }
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
            let exit_signal = self.exit_signal.clone();
            let hash_rate_thread = thread::spawn(move || {
                let address_one = address!("0xffffffffffffffffffffffffffffffffffffffff");
                let result = mine_wallet(
                    Address::ZERO,
                    address_one,
                    Some(Duration::from_secs(1)),
                    exit_signal,
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
                    .percent(std::cmp::min(
                        100,
                        (elapsed_time.as_secs() * 100)
                            .checked_div(est_time as u64)
                            .unwrap_or(100) as u16,
                    ))
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

        self.exit_popup.render(area, buf, &shared_state.theme);

        area
    }
}
