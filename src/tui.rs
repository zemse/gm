mod events;
#[macro_use]
mod traits;
pub mod app;
mod impls;
mod theme;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

use app::App;
pub use events::Event;

pub async fn run(args: Vec<String>) -> crate::Result<()> {
    let (event_tr, event_rc) = mpsc::channel::<Event>();
    let shutdown = Arc::new(AtomicBool::new(false));

    let mut terminal = ratatui::init();

    let mut app = App::new()?;
    app.cli_args(args)?;

    app.init_threads(&event_tr, &shutdown);

    while !app.exit {
        // render the view based on the controller state
        let area = app.draw(&mut terminal)?;

        // make any changes to Controller state
        let event = event_rc.recv()?;
        let result = app.handle_event(event, area, &event_tr, &shutdown).await;
        if let Err(e) = result {
            app.fatal_error_popup.set_text(e.to_string());
        }
    }

    // final render before exiting
    app.draw(&mut terminal)?;

    // signal all the threads to exit
    shutdown.store(true, Ordering::Relaxed);

    // wait for app component threads to exit
    app.exit_threads().await;

    // restore normal terminal
    ratatui::restore();

    Ok(())
}
