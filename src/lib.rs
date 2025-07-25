#[macro_use]
// pub mod actions;
pub mod alchemy;
pub mod disk;
pub mod error;
pub mod network;
pub mod tui;
pub mod utils;
pub use error::{Error, Result};

#[macro_export]
macro_rules! gm_log {
    ($($arg:tt)*) => {{
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("gm.log")
            .unwrap();

        writeln!(file, $($arg)*).unwrap();
    }};
}
