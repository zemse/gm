use alloy::primitives::Address;
use data3::{blockscout::BlockScout, network::Network};
use gm_utils::config::Config;
use std::{collections::HashSet, sync::mpsc::Sender, time::Duration};
use tokio_util::sync::CancellationToken;

use crate::AppEvent;

const DELAY: Duration = Duration::from_secs(30);

pub async fn watch_recent_addresses(
    transmitter: Sender<AppEvent>,
    shutdown_signal: CancellationToken,
) {
    let mut delay = Duration::from_secs(0);

    loop {
        tokio::select! {
            result = run_interval(delay) => {
                let _ = match result {
                    Ok(Some(addresses)) => {
                        delay = DELAY; // default duration
                        transmitter.send(AppEvent::RecentAddressesUpdate(addresses))
                    },
                    Ok(None) => Ok(()),
                    Err(error) => {
                        delay += DELAY;
                        delay *= 2; // exponential backoff in case api fails
                        transmitter.send(AppEvent::RecentAddressesUpdateError(error))
                    },
                };
            }
            _ = shutdown_signal.cancelled() => break
        };
    }
}

async fn run_interval(wait_for: Duration) -> crate::Result<Option<Vec<Address>>> {
    tokio::time::sleep(wait_for).await;

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
