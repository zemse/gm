use std::{
    fmt::{Display, Formatter},
    future::IntoFuture,
};

use alloy::primitives::{utils::format_units, Address, U256};
use inquire::Select;
use tokio::runtime::Runtime;

use crate::{
    alchemy::{Alchemy, TokensByWalletEntry},
    disk::Config,
};

pub struct Balance {
    token_address: Option<Address>,
    network: String,
    value: U256,
    symbol: String,
    decimals: u8,
    usd_price: Option<f64>,
}

impl From<TokensByWalletEntry> for Balance {
    fn from(entry: TokensByWalletEntry) -> Self {
        Self {
            token_address: Some(entry.token_address),
            network: entry.network,
            value: entry.token_balance,
            symbol: entry.token_metadata.symbol,
            decimals: entry.token_metadata.decimals,
            usd_price: entry.token_prices.first().map(|p| {
                assert_eq!(p.currency, "usd");
                p.value.parse().unwrap()
            }),
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

impl Display for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted_value = self.formatted_value();
        let usd_value = self.usd_value();
        let symbol = &self.symbol;
        if let Some(usd_value) = usd_value {
            write!(f, "{formatted_value} {symbol} (${usd_value:.2})")
        } else {
            write!(f, "{formatted_value} {symbol}")
        }
    }
}

// multichain balances
pub fn get_all_balances() {
    let result: Vec<Balance> = Runtime::new()
        .unwrap()
        .block_on(Alchemy::get_tokens_by_wallet(Config::current_account()).into_future())
        .unwrap()
        .into_iter()
        .map(|entry| entry.into())
        .filter(|entry: &Balance| entry.usd_value().map(|v| v > 0.0).unwrap_or_default())
        .collect();

    Select::new("Select asset to use", result).prompt().unwrap();

    // TODO show options to the user
    // Show coingecko page
    // Transfer to another account
    // Receive
}
