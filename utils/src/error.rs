use std::path::PathBuf;

use alloy::primitives::Address;
use serde_json::Value;
use url::Url;

use crate::reqwest::{ReqwestErrorContext, ReqwestInnerError, ReqwestStage};

pub type Result<T> = std::result::Result<T, UtilsError>;

#[derive(Debug, thiserror::Error)]
pub enum UtilsError {
    #[cfg(target_os = "macos")]
    #[error(transparent)]
    MacosError(#[from] gm_macos::Error),

    // TODO improve errors from account module
    #[error(transparent)]
    FromHexError(#[from] alloy::hex::FromHexError),

    // TODO improve errors from account module
    #[error(transparent)]
    EcdsaError(#[from] alloy::signers::k256::ecdsa::Error),

    #[error("Failed to create mnemonic. (Error: {0:?})")]
    MnemonicGenerationFailed(coins_bip39::MnemonicError),

    #[error("Failed to create signer from mnemonic. (Error: {0:?})")]
    MnemonicSignerFailed(alloy::signers::local::LocalSignerError),

    #[error("Secret not found for account {0}.")]
    SecretNotFound(alloy::primitives::Address),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error("Failed to parse Alchemy response: {0}.")]
    AlchemyResponse(&'static str),

    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("Failed to parse JSON value: {0:?}. (Error: {1:?})")]
    SerdeJsonValueParseFailed(Value, serde_json::Error),

    #[error("Network not found: {0}.")]
    NetworkNotFound(String),

    #[error("Name already exists in address book: {0}.")]
    AddressBookNameExists(String),

    #[error("Address already exists in address book: {0}.")]
    AddressBookAddressExists(Address),

    #[error("Address '{0}' is not a valid Ethereum address.")]
    InvalidAddress(String),

    #[error("String '{0}' is not a valid hex string.")]
    InvalidHexString(String),

    #[error("Current account is not loaded/selected.")]
    CurrentAccountNotSet,

    #[error("Alchemy API key not set in config, please set it.")]
    AlchemyApiKeyNotSet,

    #[error("Failed to get base directories.")]
    BaseDirsFailed,

    #[error("Failed to create directory: {0:?}. (Error: {1:?})")]
    CreateDirAllFailed(PathBuf, std::io::Error),

    #[error("Failed to read the file: {0}. (Error: {1:?})")]
    FileReadFailed(PathBuf, std::io::Error),

    #[error("Failed to write to the file: {0}. (Error: {1:?})")]
    FileWriteFailed(PathBuf, std::io::Error),

    #[error("Parsing the toml file failed: {0}. (Error: {1:?})")]
    TomlParsingFailed(PathBuf, toml::de::Error),

    #[error("Formatting to toml format failed: {0}. (Error: {1:?})")]
    TomlFormattingFailed(String, toml::ser::Error),

    #[error("Parsing the yaml file failed: {0}. (Error: {1:?})")]
    YamlParsingFailed(PathBuf, serde_yaml::Error),

    #[error("Formatting to yaml format failed: {0}. (Error: {1:?})")]
    YamlFormattingFailed(String, serde_yaml::Error),

    #[error(transparent)]
    AlloySolTypes(#[from] alloy::sol_types::Error),

    #[error("Rpc URL not found for network {network} with chain id {chain_id}. Please add it in the networks.")]
    RpcUrlNotFound { network: String, chain_id: u32 },

    #[error("Failed to parse URL: {0}. (Error: {1:?})")]
    UrlParsingFailed(String, url::ParseError),

    #[error(transparent)]
    SerdePathToError(#[from] serde_path_to_error::Error<serde_json::Error>),

    #[error("Please check your internet connection, the URL seems to be unreachable: {0}")]
    Internet(Url),

    #[error("Request '{url}' failed at stage '{stage:?}' (Error='{inner:?}', Context='{context:?}')", url = context.url)]
    ReqwestFailed {
        stage: ReqwestStage,
        context: Box<ReqwestErrorContext>,
        inner: ReqwestInnerError,
    },

    #[error("Reqwest builder missing error context, this is a bug please report it.")]
    ReqwestErrorContextMissing,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Chainlink price feed not configured for network: {0}")]
    ChainlinkPriceFeedNotConfigured(String),

    #[error("Failed to fetch latest round data from Chainlink oracle for network {network_name}. (Error: {error:?})")]
    ChainlinkLatestRoundData {
        network_name: String,
        error: Box<alloy::contract::Error>,
    },

    #[error("Failed to fetch decimals from Chainlink oracle {network_name}. (Error: {error:?})")]
    ChainlinkFetchDecimalsFailed {
        network_name: String,
        error: Box<alloy::contract::Error>,
    },

    #[error("Chainlink oracle returned a negative price on network {network_name}: {price}")]
    ChainlinkNegativePrice { network_name: String, price: String },
}

impl UtilsError {
    pub fn is_connect(&self) -> bool {
        match self {
            Self::ReqwestFailed { inner, .. } => inner.is_connect(),
            _ => false,
        }
    }
}
