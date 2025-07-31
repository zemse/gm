use std::time::Duration;

use alloy::signers::{k256::ecdsa::SigningKey, Signature};
use fusion_plus_sdk::multichain_address::MultichainAddress;
use walletconnect_sdk::wc_message::WcMessage;

use crate::{
    tui::app::widgets::{
        invite_popup::{InviteCodeClaimStatus, InviteCodeValidity},
        tx_popup::TxStatus,
    },
    utils::assets::{Asset, LightClientVerification, TokenAddress},
};

use reqwest::Error as ReqwestError;

use super::app::{
    pages::walletconnect::WalletConnectStatus,
    widgets::candle_chart::{Candle, Interval},
};

pub mod assets;
pub mod eth_price;
pub mod helios;
pub mod input;
pub mod recent_addresses;

#[derive(Debug)]
pub enum Event {
    Input(crossterm::event::KeyEvent),

    AccountChange(MultichainAddress),
    ConfigUpdate,

    EthPriceUpdate(String),
    EthPriceError(ReqwestError),

    HashRateResult(f64),
    HashRateError(String),
    VanityResult(SigningKey, usize, Duration),

    AssetsUpdate(MultichainAddress, Vec<Asset>),
    AssetsUpdateError(String, bool), // bool - whether to silence the error

    RecentAddressesUpdate(Vec<MultichainAddress>),
    RecentAddressesUpdateError(String),

    CandlesUpdate(Vec<Candle>, Interval),
    CandlesUpdateError(ReqwestError),

    TxUpdate(TxStatus),
    TxError(String),

    SignResult(Signature),
    SignError(String),

    WalletConnectStatus(WalletConnectStatus),
    WalletConnectMessage(MultichainAddress, Box<WcMessage>),
    WalletConnectError(MultichainAddress, String),

    HeliosUpdate {
        account: MultichainAddress,
        network: String,
        token_address: TokenAddress,
        status: LightClientVerification,
    },
    HeliosError(String),

    InviteCodeValidity(InviteCodeValidity),
    InviteCodeClaimStatus(InviteCodeClaimStatus),
}

impl Event {
    pub fn fmt(&self) -> String {
        format!("{self:?}")
    }

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
