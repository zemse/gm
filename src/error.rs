use std::fmt::Display;

use alloy::primitives::Address;
use serde_json::Value;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CurrentAccountNotSet,
    AlchemyApiKeyNotSet,
    DiskError(String),
    NetworkNotFound(String),
    AddressBook(&'static str),
    SecretNotFound(Address),
    InternalError(String),
    InternalErrorStr(&'static str),
    ParseIntError(Box<std::num::ParseIntError>),
    ParseFloatError(Box<std::num::ParseFloatError>),
    IoError(Box<std::io::Error>),
    FromHexError(Box<alloy::hex::FromHexError>),
    #[cfg(target_os = "macos")]
    AppleSecurityFrameworkError(Box<security_framework::base::Error>),
    InquireError(Box<inquire::InquireError>),
    AlloyEcdsaError(Box<alloy::signers::k256::ecdsa::Error>),
    TomlDeError(Box<toml::de::Error>),
    TomlSerError(Box<toml::ser::Error>),
    YamlError(Box<serde_yaml::Error>),
    ReqwestError(Box<reqwest::Error>),
    SerdeJson(Box<serde_json::Error>),
    SerdePathToError(Box<serde_path_to_error::Error<serde_json::Error>>),
    SerdeJsonWithValue(Box<serde_json::Error>, Box<Value>),
    SerdeJsonWithString(Box<serde_json::Error>, Box<String>),
    MpscRecvError(Box<std::sync::mpsc::RecvError>),
    MpscSendError(Box<std::sync::mpsc::SendError<crate::tui::Event>>),
    MpscSendError2(Box<std::sync::mpsc::SendError<crate::tui::app::pages::walletconnect::WcEvent>>),
    MnemonicError(Box<coins_bip39::MnemonicError>),
    AlloyLocalSignerError(Box<alloy::signers::local::LocalSignerError>),
    FromUtf8Error(Box<std::string::FromUtf8Error>),
    RpcError(Box<alloy::transports::RpcError<alloy::transports::TransportErrorKind>>),
    UnitsError(Box<alloy::primitives::utils::UnitsError>),
    AlloySignerError(Box<alloy::signers::Error>),
    AlloyPendingTransactionError(Box<alloy::providers::PendingTransactionError>),
    AlloyRlpError(Box<alloy::rlp::Error>),
    Abort(&'static str),
    UrlParseError(Box<url::ParseError>),
    Data3Error(Box<data3::error::Error>),
    WalletConnectError(walletconnect_sdk::Error),
}

impl Error {
    pub fn is_connect_reqwest(&self) -> bool {
        match self {
            Self::ReqwestError(error) => error.is_connect(),
            _ => false,
        }
    }
}

impl FmtError for Error {
    fn fmt_err(&self, id: &str) -> String {
        if self.is_connect_reqwest() {
            format!("Please check your internet connection - {id}: {self:#?}")
        } else {
            format!("{id}: {self:#?}")
        }
    }
}

impl FmtError for reqwest::Error {
    fn fmt_err(&self, id: &str) -> String {
        if self.is_connect() {
            format!("Please check your internet connection - {id}: {self:?}")
        } else {
            format!("{id}: {self:?}")
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AddressBook(s) => write!(f, "Error from AddressBook: {s}"),
            _ => write!(f, "{self:?}"),
        }
    }
}
impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error::InternalError(e.to_string())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseIntError(Box::new(e))
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(e: std::num::ParseFloatError) -> Self {
        Error::ParseFloatError(Box::new(e))
    }
}

impl From<alloy::hex::FromHexError> for Error {
    fn from(e: alloy::hex::FromHexError) -> Self {
        Error::FromHexError(Box::new(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(Box::new(e))
    }
}

#[cfg(target_os = "macos")]
impl From<security_framework::base::Error> for Error {
    fn from(e: security_framework::base::Error) -> Self {
        Error::AppleSecurityFrameworkError(Box::new(e))
    }
}

impl From<inquire::InquireError> for Error {
    fn from(e: inquire::InquireError) -> Self {
        Error::InquireError(Box::new(e))
    }
}

impl From<alloy::signers::k256::ecdsa::Error> for Error {
    fn from(e: alloy::signers::k256::ecdsa::Error) -> Self {
        Error::AlloyEcdsaError(Box::new(e))
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::TomlDeError(Box::new(e))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self {
        Error::TomlSerError(Box::new(e))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Error::YamlError(Box::new(e))
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::ReqwestError(Box::new(e))
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(Box::new(e))
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(e: std::sync::mpsc::RecvError) -> Self {
        Error::MpscRecvError(Box::new(e))
    }
}

impl From<std::sync::mpsc::SendError<crate::tui::Event>> for Error {
    fn from(e: std::sync::mpsc::SendError<crate::tui::Event>) -> Self {
        Error::MpscSendError(Box::new(e))
    }
}

impl From<std::sync::mpsc::SendError<crate::tui::app::pages::walletconnect::WcEvent>> for Error {
    fn from(e: std::sync::mpsc::SendError<crate::tui::app::pages::walletconnect::WcEvent>) -> Self {
        Error::MpscSendError2(Box::new(e))
    }
}

impl From<coins_bip39::MnemonicError> for Error {
    fn from(e: coins_bip39::MnemonicError) -> Self {
        Error::MnemonicError(Box::new(e))
    }
}

impl From<alloy::signers::local::LocalSignerError> for Error {
    fn from(e: alloy::signers::local::LocalSignerError) -> Self {
        Error::AlloyLocalSignerError(Box::new(e))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::FromUtf8Error(Box::new(e))
    }
}

impl From<alloy::transports::RpcError<alloy::transports::TransportErrorKind>> for Error {
    fn from(e: alloy::transports::RpcError<alloy::transports::TransportErrorKind>) -> Self {
        Error::RpcError(Box::new(e))
    }
}

impl From<alloy::primitives::utils::UnitsError> for Error {
    fn from(e: alloy::primitives::utils::UnitsError) -> Self {
        Error::UnitsError(Box::new(e))
    }
}

impl From<alloy::signers::Error> for Error {
    fn from(e: alloy::signers::Error) -> Self {
        Error::AlloySignerError(Box::new(e))
    }
}

impl From<alloy::providers::PendingTransactionError> for Error {
    fn from(e: alloy::providers::PendingTransactionError) -> Self {
        Error::AlloyPendingTransactionError(Box::new(e))
    }
}

impl From<alloy::rlp::Error> for Error {
    fn from(e: alloy::rlp::Error) -> Self {
        Error::AlloyRlpError(Box::new(e))
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::UrlParseError(Box::new(e))
    }
}

impl From<data3::error::Error> for Error {
    fn from(e: data3::error::Error) -> Self {
        Error::Data3Error(Box::new(e))
    }
}

impl From<walletconnect_sdk::Error> for Error {
    fn from(e: walletconnect_sdk::Error) -> Self {
        Error::WalletConnectError(e)
    }
}

pub trait FmtError {
    fn fmt_err(&self, id: &str) -> String;
}
