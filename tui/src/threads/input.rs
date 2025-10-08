use std::{sync::mpsc, time::Duration};

use ratatui::crossterm::{self};
use tokio_util::sync::CancellationToken;

use crate::AppEvent;

pub fn watch_input_events(tx: mpsc::Sender<AppEvent>, shutdown_signal: CancellationToken) {
    loop {
        if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
            // Send result back to main thread. If main thread has already
            // shutdown, then we will get error. Since our event is not
            // critical, we do not store it to disk.
            let _ = tx.send(AppEvent::Input(crossterm::event::read().unwrap()));
        }
        if shutdown_signal.is_cancelled() {
            break;
        }
    }
}
