pub mod act;
pub mod error;
pub mod event;
pub mod extensions;
pub mod thematize;
pub mod widgets;

#[cfg(test)]
mod tests;
#[cfg(test)]
pub mod testutils;

pub use error::RatatuiExtraError as Error;
pub type Result<T> = std::result::Result<T, Error>;
pub use widgets::*;
