use std::{
    fmt::{Display, Formatter},
    time::Duration,
};

use alloy::{
    primitives::{map::HashMap, utils::format_units, Address, U256},
    providers::Provider,
};

use crate::{
    alchemy::Alchemy, config::Config, disk_storage::DiskStorageInterface, network::NetworkStore,
};

#[derive(Default)]
pub struct AssetManager {
    assets: HashMap<Address, Option<Vec<Asset>>>,
    // prices: HashMap<String, Price>,
}

impl AssetManager {
    pub fn clear_data_for(&mut self, account: Address) {
        self.assets.remove(&account);
    }

    // Update asset
    pub fn update_assets(
        &mut self,
        account: Address,
        mut new_assets: Vec<Asset>,
    ) -> crate::Result<()> {
        let old_assets = self.assets.remove(&account).flatten().unwrap_or_default();

        for old_asset in old_assets {
            if let Some(new_asset) = new_assets.iter_mut().find(|new_asset| {
                new_asset.r#type.token_address == old_asset.r#type.token_address
                    && new_asset.r#type.network == old_asset.r#type.network
            }) {
                // If balance did not change, then carry fwd some properties
                if new_asset.value == old_asset.value {
                    new_asset.light_client_verification = old_asset.light_client_verification;
                }
            }
        }

        self.assets.insert(account, Some(new_assets));

        Ok(())
    }

    // Update light client info
    pub fn update_light_client_verification(
        &mut self,
        account: Address,
        network: String,
        token_address: TokenAddress,
        status: LightClientVerification,
    ) {
        let mut assets = self.assets.remove(&account).flatten();

        if let Some(assets) = assets.as_mut() {
            for asset in assets {
                if asset.r#type.network == network && asset.r#type.token_address == token_address {
                    asset.light_client_verification = status.clone();
                }
            }
        }

        self.assets.insert(account, assets);
    }

    // Update price
    // TODO impl

    // Get fn for render which should also factor in the price
    pub fn get_assets(&self, address: &Address) -> Option<&Vec<Asset>> {
        self.assets.get(address).and_then(|r| r.as_ref())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum TokenAddress {
    Native,
    Contract(Address),
}

impl TokenAddress {
    pub fn is_native(&self) -> bool {
        matches!(self, TokenAddress::Native)
    }

    pub fn is_contract(&self) -> bool {
        matches!(self, TokenAddress::Contract(_))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssetType {
    pub token_address: TokenAddress,
    pub network: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price: Price,
}

impl Display for AssetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.symbol, self.network)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LightClientVerification {
    Pending,
    Verified,
    Rejected,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Asset {
    pub wallet_address: Address,
    pub r#type: AssetType,
    pub value: U256,
    pub light_client_verification: LightClientVerification,
}

impl Asset {
    pub fn formatted_value(&self) -> f64 {
        let temp_formatted =
            format_units(self.value, self.r#type.decimals).expect("format_units failed");

        temp_formatted
            .parse::<f64>()
            .expect("parse into f64 failed")
    }

    pub fn usd_value(&self) -> Option<f64> {
        self.r#type
            .price
            .usd_price()
            .map(|usd_price| self.formatted_value() * usd_price)
    }
}

impl Display for Asset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted_value = self.formatted_value();
        let usd_value = self.usd_value();

        let usd_value_fmt = usd_value.map(|v| format!(" (${v:.2})")).unwrap_or_default();
        let light_client_status_fmt = match self.light_client_verification {
            LightClientVerification::Pending => "",
            LightClientVerification::Verified => " [Lightclient Verified]",
            LightClientVerification::Rejected => " [Lightclient REJECTED]",
        };

        write!(
            f,
            "{formatted_value} {symbol} {network}{usd_value_fmt}{light_client_status_fmt}",
            symbol = self.r#type.symbol,
            network = self.r#type.network
        )
    }
}

pub async fn get_all_assets(wait_for: Option<Duration>) -> crate::Result<(Address, Vec<Asset>)> {
    if let Some(dur) = wait_for {
        tokio::time::sleep(dur).await;
    }

    let config = Config::load()?;
    let wallet_address = config.get_current_account()?;

    let mut networks = NetworkStore::load()?;

    let mut balances = Vec::new();

    for entry in Alchemy::get_tokens_by_wallet(
        wallet_address,
        networks.get_alchemy_network_names(config.get_testnet_mode()),
    )
    .await?
    {
        if let Some(token_balance) = entry.token_balance {
            let network = networks
                .get_by_name(&entry.network)
                .ok_or(crate::Error::NetworkNotFound(entry.network))?;
            let asset = Asset {
                wallet_address,
                r#type: AssetType {
                    token_address: match entry.token_address {
                        Some(token_address) => TokenAddress::Contract(token_address),
                        None => TokenAddress::Native,
                    },
                    network: network.name.clone(),
                    symbol: entry.token_metadata.symbol.unwrap_or(
                        if entry.token_address.is_none() {
                            network.symbol.unwrap_or(format!("{}ETH", network.name))
                        } else {
                            "UNKNOWN".to_string()
                        },
                    ),
                    name: entry
                        .token_metadata
                        .name
                        .unwrap_or(if entry.token_address.is_none() {
                            network.name
                        } else {
                            "UNKNOWN".to_string()
                        }),
                    decimals: entry.token_metadata.decimals.unwrap_or(
                        if entry.token_address.is_none() {
                            network.native_decimals.unwrap_or(0)
                        } else {
                            0
                        },
                    ),
                    price: entry
                        .token_prices
                        .first()
                        .map(|p| {
                            assert_eq!(p.currency, "usd"); // TODO could blow up
                            Price::InUSD(p.value.parse().unwrap())
                        })
                        .unwrap_or(Price::Unknown),
                },
                value: token_balance,
                light_client_verification: LightClientVerification::Pending,
            };

            if asset.value > U256::ZERO
                && (config.get_testnet_mode()
                    || asset.usd_value().map(|v| v > 0.0).unwrap_or_default())
            // || has_token(&networks, &asset.r#type.token_address)
            {
                balances.push(asset);
            }
        }
    }

    for balance in &balances {
        if let TokenAddress::Contract(token_address) = balance.r#type.token_address {
            networks.register_token(
                &balance.r#type.network,
                token_address,
                Some(balance.r#type.symbol.as_str()),
                &balance.r#type.name,
                balance.r#type.decimals,
            );
        }
    }
    networks.save()?;

    // Fetch native balances via eth_getBalance RPC (more reliable than Alchemy API for new networks)
    for network in networks.get_iter(config.get_testnet_mode()) {
        let provider = match network.get_provider() {
            Ok(p) => p,
            Err(_) => continue, // Skip networks without working RPC
        };

        let balance = match provider.get_balance(wallet_address).await {
            Ok(b) => b,
            Err(_) => continue, // Skip on RPC errors
        };

        if balance.is_zero() {
            continue;
        }

        // Check if we already have a native balance entry from Alchemy for this network
        if let Some(existing) = balances
            .iter_mut()
            .find(|a| a.r#type.token_address.is_native() && a.r#type.network == network.name)
        {
            // TODO: Show discrepancy in the UI when Alchemy balance differs from RPC balance
            // This can help users understand why balances might appear different across sources
            // if existing.value != balance {
            //     log::warn!(
            //         "Balance discrepancy for {} on {}: Alchemy={}, RPC={}",
            //         network.symbol.as_deref().unwrap_or("ETH"),
            //         network.name,
            //         existing.value,
            //         balance
            //     );
            // }

            // Prioritize RPC balance over Alchemy (more up-to-date)
            existing.value = balance;
        } else {
            // No Alchemy entry, create a new one
            let price = if let Some(price_ticker) = &network.price_ticker {
                if price_ticker == "ETH" {
                    Price::InETH(1f64)
                } else {
                    match Alchemy::get_price(price_ticker).await {
                        Ok((price, _)) => Price::InUSD(price),
                        Err(_) => Price::Unknown,
                    }
                }
            } else {
                Price::Unknown
            };

            let asset = Asset {
                wallet_address,
                r#type: AssetType {
                    token_address: TokenAddress::Native,
                    network: network.name.clone(),
                    symbol: network.symbol.clone().unwrap_or("ETH".to_string()),
                    name: network.name.clone(),
                    decimals: network.native_decimals.unwrap_or(18),
                    price,
                },
                value: balance,
                light_client_verification: LightClientVerification::Pending,
            };

            // Apply testnet mode filter
            if config.get_testnet_mode() || asset.usd_value().map(|v| v > 0.0).unwrap_or_default() {
                balances.push(asset);
            }
        }
    }

    balances.sort_by(|a, b| {
        a.usd_value()
            .partial_cmp(&b.usd_value())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok((wallet_address, balances))
}
