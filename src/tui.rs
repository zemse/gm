mod controller;
mod events;
mod traits;
mod views;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

use controller::Controller;
use events::Event;
use views::View;

#[derive(Default)]
pub struct Tui {
    controller: Controller,
}

impl Tui {
    pub async fn run() -> crate::Result<()> {
        let (event_tr, event_rc) = mpsc::channel::<Event>();
        let shutdown = Arc::new(AtomicBool::new(false));

        let tr_input = event_tr.clone();
        let shutdown_signal = shutdown.clone();
        let thread_1 = thread::spawn(move || {
            events::input::watch_input_events(tr_input, shutdown_signal);
        });

        let tr_eth_price = event_tr.clone();
        let shutdown_signal = shutdown.clone();
        let thread_2 = tokio::spawn(async move {
            events::eth_price::watch_eth_price_change(tr_eth_price, shutdown_signal).await
        });

        let mut terminal = ratatui::init();

        let mut tui = Tui::default();

        while !tui.controller.exit() {
            // render the view based on the controller state
            View {
                exit: tui.controller.exit,
                eth_price: &tui.controller.eth_price,
                navigation: &tui.controller.navigation,
            }
            .draw(&mut terminal)?;

            // make any changes to Controller state
            tui.controller.handle(event_rc.recv()?);
        }

        // signal all the threads to exit
        shutdown.store(true, Ordering::Relaxed);

        // wait for threads to exit gracefully
        thread_1.join().unwrap();
        thread_2.await.unwrap();

        // restore normal terminal
        ratatui::restore();

        Ok(())
    }
}
