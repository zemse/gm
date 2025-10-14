use std::time::Duration;

use alloy::{
    primitives::Address,
    signers::{k256::ecdsa::SigningKey, Signature},
};
use gm_ratatui_extra::{
    candle_chart::{Candle, Interval},
    event::WidgetEvent,
};
use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseEvent,
};
use walletconnect_sdk::wc_message::WcMessage;

use gm_utils::{
    assets::{Asset, LightClientVerification, TokenAddress},
    error::UtilsError,
};

use crate::pages::{
    invite_popup::{InviteCodeClaimStatus, InviteCodeValidity},
    shell::ShellUpdate,
    tx_popup::TxStatus,
    walletconnect::WalletConnectStatus,
};

#[derive(Debug)]
pub enum AppEvent {
    Tick,

    Input(Event),

    PricesUpdate,
    PricesUpdateError(UtilsError),

    HashRateResult(f64),
    HashRateError(String),
    VanityResult(SigningKey, u64, Duration),

    AssetsUpdate(Address, Vec<Asset>),
    AssetsUpdateError(gm_utils::Error, bool), // bool - whether to silence the error

    RecentAddressesUpdate(Vec<Address>),
    RecentAddressesUpdateError(crate::Error),

    CandlesUpdate(Vec<Candle>, Interval),
    CandlesUpdateError(UtilsError),

    TxUpdate(TxStatus),
    TxError(String),

    SignResult(Address, Signature),
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

    ShellUpdate(ShellUpdate),
}

impl AppEvent {
    pub const INPUT_KEY_ENTER: AppEvent = AppEvent::Input(Event::Key(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }));

    pub fn fmt(&self) -> String {
        format!("{self:?}")
    }

    pub fn is_input(&self) -> bool {
        matches!(self, AppEvent::Input(_))
    }

    pub fn is_space_or_enter_pressed(&self) -> bool {
        self.is_char_pressed(Some(' ')) || self.is_key_pressed(KeyCode::Enter)
    }

    pub fn is_char_pressed(&self, char: Option<char>) -> bool {
        if let Some(ch) = char {
            matches!(
                self,
                AppEvent::Input(
                    Event::Key(
                        KeyEvent {
                            kind: KeyEventKind::Press,
                            code: KeyCode::Char(c),
                            ..
                        }
                    )
                ) if *c == ch,
            )
        } else {
            matches!(
                self,
                AppEvent::Input(Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    ..
                }))
            )
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        matches!(
            self,
            AppEvent::Input(
                Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code,
                modifiers: KeyModifiers::NONE,
                ..
            })) if *code == key,
        )
    }

    pub fn widget_event(&self) -> Option<WidgetEvent> {
        match self {
            AppEvent::Tick => Some(WidgetEvent::Tick),
            AppEvent::Input(event) => Some(WidgetEvent::InputEvent(event.clone())),
            _ => None,
        }
    }

    pub fn input_event(&self) -> Option<&Event> {
        match self {
            AppEvent::Input(event) => Some(event),
            _ => None,
        }
    }

    pub fn key_event(&self) -> Option<&KeyEvent> {
        match self {
            AppEvent::Input(Event::Key(key_event)) => Some(key_event),
            _ => None,
        }
    }

    pub fn mouse_event(&self) -> Option<&MouseEvent> {
        match self {
            AppEvent::Input(Event::Mouse(mouse_event)) => Some(mouse_event),
            _ => None,
        }
    }
}
