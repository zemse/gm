use super::Event;
use serde::Deserialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
    time::Duration,
};

pub async fn watch_eth_price_change(transmitter: Sender<Event>, shutdown_signal: Arc<AtomicBool>) {
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
            let _ = match query_eth_price().await {
                Ok(price) => transmitter.send(Event::EthPriceUpdate(price)),
                Err(error) => transmitter.send(Event::EthPriceError(error)),
            };
            counter = 0;
        }

        counter += thread_sleep_duration_milli;
        thread::sleep(Duration::from_millis(thread_sleep_duration_milli));
    }
}

#[derive(Deserialize)]
struct BinanceResponse {
    #[allow(dead_code)]
    symbol: String,
    price: String,
}

async fn query_eth_price() -> Result<String, reqwest::Error> {
    let url = "https://api.binance.com/api/v3/ticker/price?symbol=ETHUSDT";
    let response = reqwest::get(url).await?;
    let json = response.json::<BinanceResponse>().await?;
    Ok(json.price)
}
