use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

use crossterm::event::KeyCode;

pub fn watch_input_events(tx: mpsc::Sender<super::Event>, shutdown_signal: Arc<AtomicBool>) {
    while !shutdown_signal.load(Ordering::Relaxed) {
        #[allow(clippy::single_match)]
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                tx.send(super::Event::Input(key_event)).unwrap();
                // When we get `q` or `Esc` we are not sure if the app is
                // exiting as these keys might be useful in the application.
                // The `shutdown_signal` takes a while to be updated on the
                // main thread.
                if key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Esc {
                    // TODO improve this as this is a hacky solution
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }
            _ => {}
        }
    }
}
