use std::fmt::{Display, Formatter};

use alloy::{
    primitives::{utils::format_units, Address, U256},
    providers::Provider,
};

use crate::{
    alchemy::Alchemy,
    disk::{Config, DiskInterface},
    network::NetworkStore,
};

#[derive(Debug, Default, PartialEq)]
pub enum Price {
    #[default]
    Pending,
    Unknown,
    InETH(f64),
    InUSD(f64),
}

impl Price {
    pub fn usd_price(&self) -> Option<f64> {
        match self {
            Price::InUSD(usd_price) => Some(*usd_price),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Asset {
    #[allow(dead_code)]
    wallet_address: Address,
    token_address: Option<Address>,
    network: String,
    value: U256,
    symbol: String,
    name: String,
    decimals: u8,
    price: Price,
}

impl Asset {
    pub fn formatted_value(&self) -> f64 {
        let temp_formatted = format_units(self.value, self.decimals).expect("format_units failed");

        temp_formatted
            .parse::<f64>()
            .expect("parse into f64 failed")
    }

    pub fn usd_value(&self) -> Option<f64> {
        self.price
            .usd_price()
            .map(|usd_price| self.formatted_value() * usd_price)
    }
}

impl Display for Asset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted_value = self.formatted_value();
        let usd_value = self.usd_value();
        if let Some(usd_value) = usd_value {
            write!(
                f,
                "{formatted_value} {} {} (${usd_value:.2})",
                self.symbol, self.network
            )
        } else {
            write!(f, "{formatted_value} {} {}", self.symbol, self.network)
        }
    }
}

pub async fn get_all_assets() -> crate::Result<Vec<Asset>> {
    let config = Config::load();
    let wallet_address = config.current_account.ok_or(crate::Error::InternalError(
        "Could not find wallet address in config".to_string(),
    ))?;

    let mut networks = NetworkStore::load();

    let mut balances: Vec<Asset> = Alchemy::get_tokens_by_wallet(
        wallet_address,
        networks.get_alchemy_network_names(config.testnet_mode),
    )
    .await?
    .into_iter()
    .map(|entry| Asset {
        wallet_address,
        token_address: Some(entry.token_address),
        network: networks
            .get_by_name(&entry.network)
            .expect("must exist")
            .name
            .clone(),
        value: entry.token_balance,
        symbol: entry.token_metadata.symbol,
        name: entry.token_metadata.name,
        decimals: entry.token_metadata.decimals,
        price: entry
            .token_prices
            .first()
            .map(|p| {
                assert_eq!(p.currency, "usd");
                Price::InUSD(p.value.parse().unwrap())
            })
            .unwrap_or(Price::Unknown),
    })
    .filter(|entry: &Asset| entry.usd_value().map(|v| v > 0.0).unwrap_or_default())
    .collect();

    for balance in &balances {
        if let Some(token_address) = balance.token_address {
            networks.register_token(
                &balance.network,
                token_address,
                &balance.symbol,
                &balance.name,
                balance.decimals,
            );
        }
    }
    networks.save();

    for network in networks.get_iter(config.testnet_mode) {
        let provider = network.get_provider();

        let balance = provider.get_balance(wallet_address).await?;
        if !balance.is_zero() {
            let price = if let Some(price_ticker) = &network.price_ticker {
                if price_ticker == "ETH" {
                    Price::InETH(1f64)
                } else {
                    let (price, _) = Alchemy::get_price(price_ticker).await.expect("api failed");
                    Price::InUSD(price)
                }
            } else {
                Price::Unknown
            };

            balances.push(Asset {
                wallet_address,
                token_address: None,
                network: network.name.clone(),
                value: balance,
                symbol: network.symbol.clone().unwrap_or("ETH".to_string()),
                name: network.name.clone(),
                decimals: 18,
                price,
            });
        }
    }

    balances.sort_by(|a, b| {
        a.usd_value()
            .partial_cmp(&b.usd_value())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(balances)
}
