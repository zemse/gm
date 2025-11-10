pub mod erc20;
pub mod error;
pub mod secret;
pub mod text_truncate;
pub mod tx_meta;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
