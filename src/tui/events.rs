use std::time::Duration;

use alloy::{primitives::Address, signers::k256::ecdsa::SigningKey};

pub mod eth_price;
pub mod input;

pub enum Event {
    Input(crossterm::event::KeyEvent),
    EthPriceUpdate(String),
    AccountChange(Address),
    HashRateResult(f64),
    HashRateError(String),
    VanityResult(SigningKey, usize, Duration),
}
