mod error;
mod macos;

pub use error::MacosError as Error;
pub(crate) use error::Result;
pub use macos::Macos;
