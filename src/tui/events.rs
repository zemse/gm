use std::time::Duration;

use alloy::{primitives::Address, signers::k256::ecdsa::SigningKey};

use crate::utils::assets::Asset;

pub mod assets;
pub mod eth_price;
pub mod input;

pub enum Event {
    Input(crossterm::event::KeyEvent),
    EthPriceUpdate(String),
    AccountChange(Address),
    HashRateResult(f64),
    HashRateError(String),
    VanityResult(SigningKey, usize, Duration),
    AssetsUpdate(Vec<Asset>),
}

impl Event {
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
