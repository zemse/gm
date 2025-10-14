use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use alloy::{primitives::Address, providers::Provider, sol};
use arc_swap::ArcSwap;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::{
    alloy::StringExt,
    network::{Network, NetworkStore},
};

/// Utility to fetch prices for different assets across networks.
// TODO support Tokens
pub struct PriceManager {
    chainlinks: Vec<Chainlink>,
    prices_store: Arc<ArcSwap<Vec<Price>>>,
}

impl PriceManager {
    pub fn new(networks: &Arc<NetworkStore>) -> crate::Result<Self> {
        let mut chainlinks = Vec::new();

        for n in networks.networks.iter() {
            if n.chainlink_native_price_feed.is_some() {
                chainlinks.push(Chainlink::from_network(n)?);
            }
        }

        Ok(Self {
            chainlinks,
            prices_store: Arc::new(ArcSwap::from_pointee(Vec::new())),
        })
    }

    async fn fetch_prices(&self, delay: Option<Duration>) -> crate::Result<Vec<Price>> {
        if let Some(d) = delay {
            tokio::time::sleep(d).await;
        }

        let mut prices = Vec::new();

        let mut connect_err = None;

        match Binance::get_eth_price().await {
            Ok(price) => prices.push(price),
            Err(e) => {
                if e.is_connect() {
                    connect_err = Some(e);
                }
            }
        }

        for chainlink in &self.chainlinks {
            match chainlink.get_eth_price().await {
                Ok(price) => prices.push(price),
                Err(e) => {
                    if e.is_connect() {
                        connect_err = Some(e);
                    }
                }
            }
        }

        // Error only if we were not able to get *any* prices. If some endpoints result in connect
        // err, yet other endpoints help yield prices, then we consider it as ok.
        if prices.is_empty() {
            if let Some(connect_err) = connect_err {
                Err(connect_err)
            } else {
                Err(crate::Error::NoPrices)
            }
        } else {
            Ok(prices)
        }
    }

    /// Get latest price in a non-blocking way
    pub fn get_latest_price(&self, chain_id: u32) -> Option<Price> {
        let prices = self.prices_store.load();

        prices
            .iter()
            .filter(|price| price.chain_id == chain_id)
            .max_by_key(|price| price.updated_at)
            .cloned()
    }

    /// Refresh prices and update the store
    pub fn spawn_refresh_prices_thread<F>(
        self: &Arc<Self>,
        shutdown_signal: CancellationToken,
        on_update: F,
    ) -> tokio::task::JoinHandle<()>
    where
        F: Fn(crate::Result<()>) + Send + Sync + 'static,
    {
        let self_clone = Arc::clone(self);
        let store = Arc::clone(&self.prices_store);

        tokio::spawn(async move {
            let mut delay = None;

            loop {
                tokio::select! {
                    result = self_clone.fetch_prices(delay) => {
                        match result {
                            Ok(prices) => {
                                on_update(Ok(()));
                                store.store(Arc::new(prices));
                                delay = Some(Duration::from_secs(10));
                            }
                            Err(err) => {
                                on_update(Err(err));
                                if let Some(d) = delay.as_mut() {
                                    *d *= 2; // exponential backoff in case api fails
                                } else {
                                    delay = Some(Duration::from_secs(10));
                                }
                            }
                        }
                    },
                    _ = shutdown_signal.cancelled() => break,
                }
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct Price {
    pub usd: f64,
    updated_at: Instant,
    chain_id: u32,
}

pub struct Binance;

impl Binance {
    async fn get_eth_price() -> Result<Price, crate::Error> {
        #[derive(Deserialize, Debug)]
        struct BinanceResponse {
            price: String,
        }
        let url = "https://api.binance.com/api/v3/ticker/price?symbol=ETHUSDT";

        Ok(Price {
            usd: crate::Reqwest::get(url)?
                .receive_json::<BinanceResponse>()
                .await
                .map(|resp| resp.price)?
                .parse::<f64>()?,
            updated_at: Instant::now(),
            chain_id: 1,
        })
    }
}

pub struct Chainlink {
    network_name: String,
    rpc: String,
    chain_id: u32,
    addr: Address,
    decimals: Option<u8>,
}

impl Chainlink {
    fn from_network(network: &Network) -> crate::Result<Self> {
        let network_name = network.name.clone();
        let rpc = network.get_rpc()?;
        let addr = network
            .chainlink_native_price_feed
            .ok_or_else(|| crate::Error::ChainlinkPriceFeedNotConfigured(network_name.clone()))?;
        Ok(Chainlink {
            network_name,
            rpc,
            chain_id: network.chain_id,
            addr,
            decimals: network.chainlink_native_price_feed_decimals,
        })
    }

    async fn get_eth_price(&self) -> Result<Price, crate::Error> {
        let data = self
            .contract()?
            .latestRoundData()
            .call()
            .await
            .map_err(|error| crate::Error::ChainlinkLatestRoundData {
                network_name: self.rpc.clone(),
                error: Box::new(error),
            })?;

        if data.answer.is_negative() {
            return Err(crate::Error::ChainlinkNegativePrice {
                network_name: self.network_name.clone(),
                price: data.answer.to_string(),
            });
        }

        let decimals = match self.decimals {
            Some(d) => d,
            None => self.contract()?.decimals().call().await.map_err(|error| {
                crate::Error::ChainlinkFetchDecimalsFailed {
                    network_name: self.rpc.clone(),
                    error: Box::new(error),
                }
            })?,
        };

        let mut price = data.answer.into_raw().to_string().parse::<f64>()?;
        if decimals > 0 {
            price /= 10f64.powi(decimals as i32);
        }

        Ok(Price {
            usd: price,
            updated_at: Instant::now(),
            chain_id: self.chain_id,
        })
    }

    fn contract(
        &self,
    ) -> crate::Result<AggregatorV3Interface::AggregatorV3InterfaceInstance<impl Provider + use<'_>>>
    {
        Ok(AggregatorV3Interface::new(
            self.addr,
            self.rpc.to_alloy_provider()?,
        ))
    }
}

sol! {
    #[sol(rpc)]
    interface AggregatorV3Interface {
        function decimals() external view returns (uint8);
        function latestRoundData()
            external
            view
            returns (
                uint80 roundId,
                int256 answer,
                uint256 startedAt,
                uint256 updatedAt,
                uint80 answeredInRound
            );
    }
}
