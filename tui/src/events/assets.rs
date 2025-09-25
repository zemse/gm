use gm_utils::assets::get_all_assets;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    time::Duration,
};

use super::Event;

const DELAY: Duration = Duration::from_secs(10);

pub async fn watch_assets(transmitter: Sender<Event>, shutdown_signal: Arc<AtomicBool>) {
    let mut delay = Duration::from_secs(0);

    loop {
        tokio::select! {
            result = {
                tokio::time::sleep(delay).await;
                get_all_assets()
            } => {
                let _ = match result {
                    Ok((wallet_address, assets)) => {
                        delay = DELAY; // default duration
                        transmitter.send(Event::AssetsUpdate(wallet_address, assets))
                    }
                    Err(error) => {
                        let silence_error = matches!(
                            error,
                            gm_utils::Error::CurrentAccountNotSet
                                | gm_utils::Error::AlchemyApiKeyNotSet
                        );
                        delay += DELAY; // exponential backoff in case api fails
                        delay *= 2; // exponential backoff in case api fails
                        transmitter.send(Event::AssetsUpdateError(error, silence_error))
                    }
                };
            }
            _ = async {
                while !shutdown_signal.load(Ordering::Relaxed) {
                    tokio::task::yield_now().await;
                }
            } => {
                break;
            }
        };
    }
}
