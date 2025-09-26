use gm_utils::assets::get_all_assets;
use std::{sync::mpsc::Sender, time::Duration};
use tokio_util::sync::CancellationToken;

use super::Event;

const DELAY_ZERO: Duration = Duration::from_secs(0);
const DELAY: Duration = Duration::from_secs(10);

pub async fn watch_assets(transmitter: Sender<Event>, shutdown_signal: CancellationToken) {
    let mut delay = Duration::from_secs(0);

    loop {
        tokio::select! {
            result = get_all_assets(Some(delay)) => {
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
                        if delay == DELAY_ZERO {
                            delay += DELAY; // backoff in case api fails
                        } else {
                            delay *= 2; // exponential backoff in case api fails
                        }
                        transmitter.send(Event::AssetsUpdateError(error, silence_error))
                    }
                };
            }
            _ = shutdown_signal.cancelled() => break
        };
    }
}
