use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    time::Duration,
};

use crate::{error::FmtError, utils::assets::get_all_assets};

use super::Event;

pub async fn watch_assets(transmitter: Sender<Event>, shutdown_signal: Arc<AtomicBool>) {
    // Query interval is the API query delay, however to prevent blocking at
    // the thread::sleep, which will cause delayed processing of shutdown_signal.
    // To prevent this, we check shutdown_signal at shorter intervals while
    // making API calls at a longer duration.
    let query_interval_milli = 2000;
    let thread_sleep_duration_milli = 100;

    let mut counter = query_interval_milli;
    while !shutdown_signal.load(Ordering::Relaxed) {
        if counter >= query_interval_milli {
            // Send result back to main thread. If main thread has already
            // shutdown, then we will get error. Since our event is not
            // critical, we do not store it to disk.
            let _ = match get_all_assets().await {
                Ok(assets) => transmitter.send(Event::AssetsUpdate(assets)),
                Err(error) => transmitter.send(Event::AssetsUpdateError(error.fmt_err())),
            };
            counter = 0;
        }

        counter += thread_sleep_duration_milli;
        tokio::time::sleep(Duration::from_millis(thread_sleep_duration_milli)).await;
    }
}
