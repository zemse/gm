mod error;
pub use error::{Error, Result};

mod app;
#[cfg(feature = "demo")]
mod demo;
mod events;
pub mod pages;
mod theme;
mod traits;
mod widgets;

pub use app::{App, MainMenuItem};
pub use events::AppEvent;
