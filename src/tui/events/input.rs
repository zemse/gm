use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

use crossterm::event::KeyCode;

pub fn watch_input_events(tx: mpsc::Sender<super::Event>, shutdown_signal: Arc<AtomicBool>) {
    while !shutdown_signal.load(Ordering::Relaxed) {
        #[allow(clippy::single_match)]
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                tx.send(super::Event::Input(key_event)).unwrap();
                if key_event.code == KeyCode::Char('q') {
                    break;
                }
            }
            _ => {}
        }
    }
}
