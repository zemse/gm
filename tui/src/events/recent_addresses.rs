use alloy::primitives::Address;
use data3::{blockscout::BlockScout, network::Network};
use gm_utils::config::Config;
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    time::Duration,
};

use super::Event;

const DELAY: Duration = Duration::from_secs(10);

pub async fn watch_recent_addresses(transmitter: Sender<Event>, shutdown_signal: Arc<AtomicBool>) {
    let mut delay = Duration::from_secs(0);

    loop {
        tokio::select! {
            result = {
                tokio::time::sleep(delay).await;
                run_interval()
            } => {
                let _ = match result {
                    Ok(Some(addresses)) => {
                        delay = DELAY; // default duration
                        transmitter.send(Event::RecentAddressesUpdate(addresses))
                    },
                    Ok(None) => Ok(()),
                    Err(error) => {
                        delay += DELAY;
                        delay *= 2; // exponential backoff in case api fails
                        transmitter.send(Event::RecentAddressesUpdateError(error))
                    },
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

async fn run_interval() -> crate::Result<Option<Vec<Address>>> {
    let Ok(current_address) = Config::current_account() else {
        return Ok(None);
    };

    // TODO support all networks
    let result = BlockScout::from_network(Network::ArbitrumMainnet)
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
