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
                // When we want to quit from our main thread, we want to
                // gracefully quit the this thread, however it is blocked on the
                // `event::read()` above. It needs a key press to be able to
                // check the shutdown signal.
                //
                // Exit is only triggered by `control + c` and `ESC` keys. Hence we are
                // adding a hacky solution to the above problem. The
                // `shutdown_signal` takes a while to be updated on the main
                // thread, so we wait for a moment before letting the execution
                // go to the while loop condition check.
                if key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Esc {
                    // TODO improve this as this is a hacky solution
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            _ => {}
        }
    }
}
