use std::time::Duration;

use alloy::{
    primitives::{Address, FixedBytes},
    signers::k256::ecdsa::SigningKey,
};

use crate::utils::assets::Asset;

use reqwest::Error as ReqwestError;

use super::app::{
    pages::transaction::TxStatus,
    widgets::candle_chart::{Candle, Interval},
};

pub mod assets;
pub mod eth_price;
pub mod input;

#[derive(Debug)]
pub enum Event {
    Input(crossterm::event::KeyEvent),

    AccountChange(Address),
    ConfigUpdated,

    EthPriceUpdate(String),
    EthPriceError(ReqwestError),

    HashRateResult(f64),
    HashRateError(String),
    VanityResult(SigningKey, usize, Duration),

    AssetsUpdate(Vec<Asset>),
    AssetsUpdateError(String, bool), // bool - whether to silence the error

    CandlesUpdate(Vec<Candle>, Interval),
    CandlesUpdateError(ReqwestError),

    TxSubmitResult(FixedBytes<32>),
    TxSubmitError(String),

    TxStatus(TxStatus),
    TxStatusError(String),
}

impl Event {
    pub fn is_input(&self) -> bool {
        matches!(self, Event::Input(_))
    }

    pub fn is_space_or_enter_pressed(&self) -> bool {
        self.is_char_pressed(Some(' ')) || self.is_key_pressed(crossterm::event::KeyCode::Enter)
    }

    pub fn is_char_pressed(&self, char: Option<char>) -> bool {
        if let Some(ch) = char {
            matches!(
                self,
                Event::Input(crossterm::event::KeyEvent {
                    kind: crossterm::event::KeyEventKind::Press,
                    code: crossterm::event::KeyCode::Char(c),
                    ..
                }) if *c == ch
            )
        } else {
            matches!(
                self,
                Event::Input(crossterm::event::KeyEvent {
                    kind: crossterm::event::KeyEventKind::Press,
                    ..
                })
            )
        }
    }

    pub fn is_key_pressed(&self, key: crossterm::event::KeyCode) -> bool {
        matches!(
            self,
            Event::Input(crossterm::event::KeyEvent {
                kind: crossterm::event::KeyEventKind::Press,
                code,
                modifiers: crossterm::event::KeyModifiers::NONE,
                ..
            }) if *code == key
        )
    }
}
