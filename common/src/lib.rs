pub mod error;
pub mod secret;
pub mod text_truncate;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
