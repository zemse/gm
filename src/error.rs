use std::fmt::Display;

use alloy::primitives::Address;
use serde_json::Value;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CurrentAccountNotSet,
    AlchemyApiKeyNotSet,
    NetworkNotFound(String),
    AddressBook(String),
    SecretNotFound(Address),
    InternalError(String),
    InternalErrorStr(&'static str),
    ParseFloatError(std::num::ParseFloatError),
    IoError(std::io::Error),
    FromHexError(alloy::hex::FromHexError),
    #[cfg(target_os = "macos")]
    AppleSecurityFrameworkError(security_framework::base::Error),
    InquireError(inquire::InquireError),
    AlloyEcdsaError(alloy::signers::k256::ecdsa::Error),
    TomlDeError(toml::de::Error),
    TomlSerError(toml::ser::Error),
    YamlError(serde_yaml::Error),
    ReqwestError(Box<reqwest::Error>),
    SerdeJson(Box<serde_json::Error>),
    SerdePathToError(Box<serde_path_to_error::Error<serde_json::Error>>),
    SerdeJsonWithValue(Box<serde_json::Error>, Box<Value>),
    SerdeJsonWithString(Box<serde_json::Error>, String),
    MpscRecvError(std::sync::mpsc::RecvError),
    MpscSendError(std::sync::mpsc::SendError<crate::tui::Event>),
    MnemonicError(coins_bip39::MnemonicError),
    AlloyLocalSignerError(alloy::signers::local::LocalSignerError),
    FromUtf8Error(std::string::FromUtf8Error),
    RpcError(Box<alloy::transports::RpcError<alloy::transports::TransportErrorKind>>),
    UnitsError(alloy::primitives::utils::UnitsError),
    AlloySignerError(alloy::signers::Error),
    AlloyPendingTransactionError(alloy::providers::PendingTransactionError),
    Abort(&'static str),
    UrlParseError(url::ParseError),
    Data3Error(data3::error::Error),
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

impl From<std::num::ParseFloatError> for Error {
    fn from(e: std::num::ParseFloatError) -> Self {
        Error::ParseFloatError(e)
    }
}

impl From<alloy::hex::FromHexError> for Error {
    fn from(e: alloy::hex::FromHexError) -> Self {
        Error::FromHexError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

#[cfg(target_os = "macos")]
impl From<security_framework::base::Error> for Error {
    fn from(e: security_framework::base::Error) -> Self {
        Error::AppleSecurityFrameworkError(e)
    }
}

impl From<inquire::InquireError> for Error {
    fn from(e: inquire::InquireError) -> Self {
        Error::InquireError(e)
    }
}

impl From<alloy::signers::k256::ecdsa::Error> for Error {
    fn from(e: alloy::signers::k256::ecdsa::Error) -> Self {
        Error::AlloyEcdsaError(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::TomlDeError(e)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self {
        Error::TomlSerError(e)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Error::YamlError(e)
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
        Error::MpscRecvError(e)
    }
}

impl From<std::sync::mpsc::SendError<crate::tui::Event>> for Error {
    fn from(e: std::sync::mpsc::SendError<crate::tui::Event>) -> Self {
        Error::MpscSendError(e)
    }
}

impl From<coins_bip39::MnemonicError> for Error {
    fn from(e: coins_bip39::MnemonicError) -> Self {
        Error::MnemonicError(e)
    }
}

impl From<alloy::signers::local::LocalSignerError> for Error {
    fn from(e: alloy::signers::local::LocalSignerError) -> Self {
        Error::AlloyLocalSignerError(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::FromUtf8Error(e)
    }
}

impl From<alloy::transports::RpcError<alloy::transports::TransportErrorKind>> for Error {
    fn from(e: alloy::transports::RpcError<alloy::transports::TransportErrorKind>) -> Self {
        Error::RpcError(Box::new(e))
    }
}

impl From<alloy::primitives::utils::UnitsError> for Error {
    fn from(e: alloy::primitives::utils::UnitsError) -> Self {
        Error::UnitsError(e)
    }
}

impl From<alloy::signers::Error> for Error {
    fn from(e: alloy::signers::Error) -> Self {
        Error::AlloySignerError(e)
    }
}

impl From<alloy::providers::PendingTransactionError> for Error {
    fn from(e: alloy::providers::PendingTransactionError) -> Self {
        Error::AlloyPendingTransactionError(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::UrlParseError(e)
    }
}

impl From<data3::error::Error> for Error {
    fn from(e: data3::error::Error) -> Self {
        Error::Data3Error(e)
    }
}

pub trait FmtError {
    fn fmt_err(&self, id: &str) -> String;
}
