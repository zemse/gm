use std::io;

use alloy::{primitives::Address, rpc::types::TransactionRequest};
use serde_json::Value;
use walletconnect_sdk::wc_message::WcMessage;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(target_os = "macos")]
    #[error(transparent)]
    MacosError(#[from] gm_macos::Error),

    #[error(transparent)]
    UtilsError(#[from] gm_utils::Error),

    #[error(transparent)]
    RatatuiExtraError(#[from] gm_ratatui_extra::Error),

    #[error("No current account is set. Please create a new account or load existing.")]
    CurrentAccountNotSet,

    #[error("Alchemy API key is not set. Please set it in the config file.")]
    AlchemyApiKeyNotSet,

    #[error("Network with name '{0}' not found in your config file.")]
    NetworkNotFound(String),

    #[error("AddressBook error: {0}.")]
    AddressBookEntryNotFound(Address, String),

    #[error("Address book entry is invalid.")]
    AddressBookEntryIsInvalid,

    #[error("Address book entry with name '{0}' already exists.")]
    SecretNotFound(Address),

    #[error("Asset is not selected.")]
    AssetNotSelected,

    #[error("Assets not found for owner {0}.")]
    AssetsNotFound(Address),

    #[error("Value for '{0}' cannot be empty.")]
    CannotBeEmpty(String),

    #[error("EIP-712 Typed Data is missing field: {0}.")]
    TypedDataMissingField(String),

    #[error("Transaction type is not specified in the request: {0:?}.")]
    TxTypeNotSpecified(Box<TransactionRequest>),

    #[error("Transaction type is not EIP-1559.")]
    TxTypeIsNotEip1559,

    #[error("WalletConnect Session request not found at index {0}, num requests: {1}.")]
    SessionRequestNotFound(usize, usize),

    #[error("Not a WalletConnect session request.")]
    NotSessionRequest,

    #[error("Chain ID '{0}' is not a valid EIP-155 chain ID.")]
    ChainIdStripEip155Failed(String),

    #[error("Chain ID '{0}' is not a valid u32.")]
    ChainIdParseFailed(String),

    #[error("Method is not handled. Request: {0:?}")]
    MethodUnhandled(Box<WcMessage>),

    #[error("Not a proposal, please report this bug.")]
    ProposalNotFound,

    #[error("Transmitter 2 channel not created.")]
    Transmitter2NotCreated,

    #[error("Poisoned lock, please restart gm.")]
    Poisoned(String),

    #[error("Draw failed: {0}")]
    Draw(std::io::Error),

    #[error("Unknown Theme: {0}")]
    UnknownTheme(String),

    #[error("Failed to generated EIP-712 typed hash. (Error: {0:?})")]
    Eip712Error(alloy::dyn_abi::Error),

    #[error("Spawn process failed. (Error: {0})")]
    SpawnFailed(io::Error),

    #[error("Failed to write to child stdin. (Error: {0})")]
    StdinWriteFailed(io::Error),

    #[error("Child stdout not available.")]
    StdoutNotAvailable,

    #[error("Failed to read from child stdout. (Error: {0})")]
    StdoutReadFailed(String),

    #[error("Child stderr not available.")]
    StderrNotAvailable,

    #[error("Failed to read from child stderr. (Error: {0})")]
    StderrReadFailed(String),

    #[error("Failed to wait for process exit. (Error: {0})")]
    ProcessExitWaitFailed(String),

    #[error("RPC Proxy thread crashed for {1}. (Error: {0})")]
    RpcProxyThreadCrashed(gm_rpc_proxy::Error, String),

    #[error("RefCall Option value {0} is already taken")]
    ValueAlreadyTaken(&'static str),

    #[error("Shell environment variables are not set.")]
    ShellEnvVarsNotSet,

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Tokio oneshot send failed")]
    OneshotSendFailed,

    #[error(
        "The request asked to perform operation using address {asked}, however current working address is {current}."
    )]
    RequestAsksForDifferentAddress { asked: Address, current: Address },

    #[error(transparent)]
    ParseFloatError(Box<std::num::ParseFloatError>),
    #[error(transparent)]
    IoError(Box<std::io::Error>),
    #[error(transparent)]
    FromHexError(Box<alloy::hex::FromHexError>),
    #[error("Alloy Ecdsa: {0}")]
    AlloyEcdsaError(Box<alloy::signers::k256::ecdsa::Error>),
    #[error("Serde Json Error: {0}")]
    SerdeJson(Box<serde_json::Error>),
    #[error("Serde Json Error with Value: {0}, {1}")]
    SerdeJsonWithValue(Box<serde_json::Error>, Box<Value>),
    #[error("Serde Json Error with String: {0}, {1}")]
    SerdeJsonWithString(Box<serde_json::Error>, Box<String>),
    #[error("Mpsc Recv Error: {0}")]
    MpscRecvError(Box<std::sync::mpsc::RecvError>),
    #[error("Mpsc Send Error: {0}")]
    MpscSendError(Box<std::sync::mpsc::SendError<crate::Event>>),
    #[error("Mpsc Send Error 2: {0}")]
    MpscSendError2(Box<std::sync::mpsc::SendError<crate::pages::walletconnect::WcEvent>>),
    #[error("Alloy Local Signer Error: {0}")]
    AlloyLocalSignerError(Box<alloy::signers::local::LocalSignerError>),
    #[error("FromUtf8 Error: {0}")]
    FromUtf8Error(Box<std::string::FromUtf8Error>),
    #[error("Rpc Error: {0}")]
    RpcError(Box<alloy::transports::RpcError<alloy::transports::TransportErrorKind>>),
    #[error("Units Error: {0}")]
    UnitsError(Box<alloy::primitives::utils::UnitsError>),
    #[error("Alloy Signer Error: {0}")]
    AlloySignerError(Box<alloy::signers::Error>),
    #[error("Pending Transaction Error: {0}")]
    AlloyPendingTransactionError(Box<alloy::providers::PendingTransactionError>),
    #[error("RLP Error: {0}")]
    AlloyRlpError(Box<alloy::rlp::Error>),
    #[error("Sol Types Error: {0}")]
    AlloySolTypesError(alloy::sol_types::Error),
    #[error("Internal error: {0}")]
    Abort(&'static str),
    #[error("Internal error: {0}")]
    UrlParseError(Box<url::ParseError>),
    #[error("Reqwest error: {0:?}")]
    Data3Error(Box<data3::error::Error>),
    #[error("Reqwest error: {0:?}")]
    WalletConnectError(walletconnect_sdk::Error),
    #[error("Eyre error: {0}")]
    EyreError(Box<eyre::Report>),
}

impl Error {
    pub fn is_connect_reqwest(&self) -> bool {
        match self {
            Self::UtilsError(error) => error.is_connect(),
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

impl From<std::sync::mpsc::SendError<crate::Event>> for Error {
    fn from(e: std::sync::mpsc::SendError<crate::Event>) -> Self {
        Error::MpscSendError(Box::new(e))
    }
}

impl From<std::sync::mpsc::SendError<crate::pages::walletconnect::WcEvent>> for Error {
    fn from(e: std::sync::mpsc::SendError<crate::pages::walletconnect::WcEvent>) -> Self {
        Error::MpscSendError2(Box::new(e))
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

impl From<alloy::sol_types::Error> for Error {
    fn from(e: alloy::sol_types::Error) -> Self {
        Error::AlloySolTypesError(e)
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

impl From<eyre::Report> for Error {
    fn from(e: eyre::Report) -> Self {
        Error::EyreError(Box::new(e))
    }
}

pub trait FmtError {
    fn fmt_err(&self, id: &str) -> String;
}
