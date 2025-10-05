use std::io::stdout;

use ratatui::crossterm::{event::DisableMouseCapture, execute, terminal::disable_raw_mode};

/// Sets a panic hook to provide better error messages and encourage users to report bugs.
pub fn set() {
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), DisableMouseCapture);

        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            info.to_string()
        };

        if let Some(loc) = info.location() {
            eprintln!("Panic: {msg:?} at {}:{}", loc.file(), loc.line());
        } else {
            eprintln!("Panic: {msg:?}");
        }

        eprintln!("This is a bug! Please report it at https://github.com/zemse/gm/issues/new");
    }));
}
