mod error;
pub use error::{Error, Result};

mod app;
mod events;
pub mod pages;
mod theme;
mod traits;
mod widgets;

pub use app::App;
pub use events::Event;
