mod error;
pub use error::{Error, Result};

mod app;
#[cfg(feature = "demo")]
mod demo;
mod events;
pub mod pages;
mod post_handle_event;
mod theme;
mod threads;
mod traits;
mod widgets;

pub use app::{App, Focus, MainMenuItem};
pub use events::AppEvent;
