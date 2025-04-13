use super::Event;
use serde::Deserialize;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread,
    time::Duration,
};

pub async fn watch_eth_price_change(transmitter: Sender<Event>, shutdown_signal: Arc<AtomicBool>) {
    while !shutdown_signal.load(Ordering::Relaxed) {
        // Send GET request
        if let Ok(price) = query_eth_price().await {
            transmitter.send(Event::EthPriceUpdate(price)).unwrap();
        }

        thread::sleep(Duration::from_secs(2));
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
