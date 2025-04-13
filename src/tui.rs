mod events;
mod traits;
mod views;

use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

use crossterm::event::{KeyCode, KeyEventKind};
use events::Event;
use views::View;

#[derive(Default)]
pub struct Tui {
    exit: bool,
    eth_price: Option<String>,
}

impl Tui {
    pub async fn run(mut self) -> io::Result<()> {
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

        while !self.exit {
            // make any changes to App state
            #[allow(clippy::single_match)]
            match event_rc.recv().unwrap() {
                Event::Input(key_event) => {
                    if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('q')
                    {
                        self.exit = true
                    }
                    if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('e')
                    {
                        self.eth_price = Some("tempxx".to_string());
                    }
                }
                Event::EthPriceUpdate(eth_price) => {
                    self.eth_price = Some(eth_price);
                }
            };

            // then render the views
            View {
                exit: self.exit,
                eth_price: &self.eth_price,
            }
            .draw(&mut terminal)?;
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
