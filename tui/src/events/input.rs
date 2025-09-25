use std::{sync::mpsc, time::Duration};

use ratatui::crossterm::{self, event::Event};
use tokio_util::sync::CancellationToken;

pub fn watch_input_events(tx: mpsc::Sender<super::Event>, shutdown_signal: CancellationToken) {
    loop {
        if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
            #[allow(clippy::single_match)]
            match crossterm::event::read().unwrap() {
                Event::Key(key_event) => {
                    // Send result back to main thread. If main thread has already
                    // shutdown, then we will get error. Since our event is not
                    // critical, we do not store it to disk.
                    let _ = tx.send(super::Event::Input(key_event));
                }
                _ => {}
            }
        }
        if shutdown_signal.is_cancelled() {
            break;
        }
    }
}
