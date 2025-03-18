use std::{
    fmt::{Display, Formatter},
    future::IntoFuture,
};

use alloy::primitives::{utils::format_units, Address, U256};
use inquire::Select;
use tokio::runtime::Runtime;

use crate::{
    alchemy::Alchemy,
    disk::{Config, DiskInterface},
    network::NetworkStore,
};

#[derive(Debug)]
pub struct Balance {
    wallet_address: Address,
    token_address: Option<Address>,
    network: String,
    value: U256,
    symbol: String,
    name: String,
    decimals: u8,
    usd_price: Option<f64>,
}

impl Display for Balance {
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

impl Balance {
    pub fn formatted_value(&self) -> f64 {
        format_units(self.value, self.decimals)
            .expect("format_units failed")
            .parse::<f64>()
            .expect("parse into f64 failed")
    }

    pub fn usd_value(&self) -> Option<f64> {
        self.usd_price
            .map(|usd_price| self.formatted_value() * usd_price)
    }
}

// multichain balances
pub fn get_all_balances() {
    let wallet_address = Config::current_account();

    let mut networks = NetworkStore::load();

    let balances: Vec<Balance> = Runtime::new()
        .unwrap()
        .block_on(
            Alchemy::get_tokens_by_wallet(wallet_address, networks.get_alchemy_network_names())
                .into_future(),
        )
        .unwrap()
        .into_iter()
        .map(|entry| Balance {
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
            usd_price: entry.token_prices.first().map(|p| {
                assert_eq!(p.currency, "usd");
                p.value.parse().unwrap()
            }),
        })
        .filter(|entry: &Balance| entry.usd_value().map(|v| v > 0.0).unwrap_or_default())
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

    Select::new("Select asset to use", balances)
        .prompt()
        .unwrap();

    // TODO show options to the user
    // Show coingecko page
    // Transfer to another account
    // Receive
}
