use std::time::Duration;

use alloy::{
    primitives::Address,
    signers::{k256::ecdsa::SigningKey, Signature},
};
use gm_ratatui_extra::candle_chart::{Candle, Interval};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use walletconnect_sdk::wc_message::WcMessage;

use gm_utils::assets::{Asset, LightClientVerification, TokenAddress};

use reqwest::Error as ReqwestError;

use crate::pages::{
    invite_popup::{InviteCodeClaimStatus, InviteCodeValidity},
    tx_popup::TxStatus,
    walletconnect::WalletConnectStatus,
};

pub mod assets;
pub mod eth_price;
pub mod helios;
pub mod input;
pub mod recent_addresses;

#[derive(Debug)]
pub enum Event {
    Input(KeyEvent),

    AccountChange(Address),
    ConfigUpdate,

    EthPriceUpdate(String),
    EthPriceError(ReqwestError),

    HashRateResult(f64),
    HashRateError(String),
    VanityResult(SigningKey, usize, Duration),

    AssetsUpdate(Address, Vec<Asset>),
    AssetsUpdateError(gm_utils::Error, bool), // bool - whether to silence the error

    RecentAddressesUpdate(Vec<Address>),
    RecentAddressesUpdateError(crate::Error),

    CandlesUpdate(Vec<Candle>, Interval),
    CandlesUpdateError(ReqwestError),

    TxUpdate(TxStatus),
    TxError(String),

    SignResult(Signature),
    SignError(String),

    WalletConnectStatus(WalletConnectStatus),
    WalletConnectMessage(Address, Box<WcMessage>),
    WalletConnectError(Address, String),

    HeliosUpdate {
        account: Address,
        network: String,
        token_address: TokenAddress,
        status: LightClientVerification,
    },
    HeliosError(String),

    InviteCodeValidity(InviteCodeValidity),
    InviteCodeClaimStatus(InviteCodeClaimStatus),
    InviteError(String),
}

impl Event {
    pub fn fmt(&self) -> String {
        format!("{self:?}")
    }

    pub fn is_input(&self) -> bool {
        matches!(self, Event::Input(_))
    }

    pub fn is_space_or_enter_pressed(&self) -> bool {
        self.is_char_pressed(Some(' ')) || self.is_key_pressed(KeyCode::Enter)
    }

    pub fn is_char_pressed(&self, char: Option<char>) -> bool {
        if let Some(ch) = char {
            matches!(
                self,
                Event::Input(KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Char(c),
                    ..
                }) if *c == ch
            )
        } else {
            matches!(
                self,
                Event::Input(KeyEvent {
                    kind: KeyEventKind::Press,
                    ..
                })
            )
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        matches!(
            self,
            Event::Input(KeyEvent {
                kind: KeyEventKind::Press,
                code,
                modifiers: KeyModifiers::NONE,
                ..
            }) if *code == key
        )
    }

    pub fn key_event(&self) -> Option<&KeyEvent> {
        if let Event::Input(key_event) = self {
            Some(key_event)
        } else {
            None
        }
    }
}
