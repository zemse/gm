use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    time::Duration,
};

use alloy::primitives::Address;

use crate::{
    blockscout::{BlockScout, BlockScoutNetwork},
    disk::Config,
    error::FmtError,
};

use super::Event;

pub async fn watch_recent_addresses(transmitter: Sender<Event>, shutdown_signal: Arc<AtomicBool>) {
    // Query interval is the API query delay, however to prevent blocking at
    // the thread::sleep, which will cause delayed processing of shutdown_signal.
    // To prevent this, we check shutdown_signal at shorter intervals while
    // making API calls at a longer duration.
    let query_interval_milli = 10000;
    let thread_sleep_duration_milli = 100;

    let mut counter = query_interval_milli;
    while !shutdown_signal.load(Ordering::Relaxed) {
        if counter >= query_interval_milli {
            // Send result back to main thread. If main thread has already
            // shutdown, then we will get error. Since our event is not
            // critical, we do not store it to disk.
            let result = get_recent_addresses().await;

            let _ = match result {
                Ok(Some(addresses)) => transmitter.send(Event::RecentAddressesUpdate(addresses)),
                Ok(None) => Ok(()),
                Err(error) => transmitter.send(Event::RecentAddressesUpdateError(
                    error.fmt_err("RecentAddressesUpdateError"),
                )),
            };
            counter = 0;
        }

        counter += thread_sleep_duration_milli;
        tokio::time::sleep(Duration::from_millis(thread_sleep_duration_milli)).await;
    }
}

async fn get_recent_addresses() -> crate::Result<Option<Vec<Address>>> {
    let Some(current_address) = Config::current_account() else {
        return Ok(None);
    };

    // TODO support all networks
    let result = BlockScout {
        network: BlockScoutNetwork::Arbitrum,
    }
    .address_transactions(current_address)
    .await?;

    Ok(Some(
        result
            .items
            .iter()
            .filter_map(|tx| {
                if tx.from.hash == current_address.to_string() {
                    tx.to.hash.parse().ok()
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect(),
    ))
}
