mod events;
#[macro_use]
mod traits;
mod app;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

use app::App;
pub use events::Event;

pub async fn run() -> crate::Result<()> {
    let (event_tr, event_rc) = mpsc::channel::<Event>();
    let shutdown = Arc::new(AtomicBool::new(false));

    let mut terminal = ratatui::init();

    let mut app = App::default();

    app.init_threads(&event_tr, &shutdown);

    while !app.exit {
        // render the view based on the controller state
        app.draw(&mut terminal)?;

        // make any changes to Controller state
        app.handle_event(event_rc.recv()?, &event_tr, &shutdown)
            .await?;
    }

    // signal all the threads to exit
    shutdown.store(true, Ordering::Relaxed);

    // wait for app component threads to exit
    app.exit_threads().await;

    // restore normal terminal
    ratatui::restore();

    Ok(())
}
