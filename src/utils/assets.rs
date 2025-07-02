use std::fmt::{Display, Formatter};

use alloy::primitives::{utils::format_units, Address, U256};

use crate::{
    alchemy::Alchemy,
    disk::{Config, DiskInterface},
    network::NetworkStore,
};

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
pub struct Asset {
    #[allow(dead_code)]
    pub wallet_address: Address,
    pub r#type: AssetType,
    pub value: U256,
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
        if let Some(usd_value) = usd_value {
            write!(
                f,
                "{formatted_value} {} {} (${usd_value:.2})",
                self.r#type.symbol, self.r#type.network
            )
        } else {
            write!(
                f,
                "{formatted_value} {} {}",
                self.r#type.symbol, self.r#type.network
            )
        }
    }
}

#[allow(dead_code)]
fn has_token(networks: &NetworkStore, token_address: &TokenAddress) -> bool {
    match token_address {
        TokenAddress::Native => false,
        TokenAddress::Contract(address) => networks.has_token(address),
    }
}

pub async fn get_all_assets() -> crate::Result<Vec<Asset>> {
    let config = Config::load()?;
    let wallet_address = config
        .current_account
        .ok_or(crate::Error::CurrentAccountNotSet)?;

    let mut networks = NetworkStore::load()?;

    let mut balances = Vec::new();

    for entry in Alchemy::get_tokens_by_wallet(
        wallet_address,
        networks.get_alchemy_network_names(config.testnet_mode),
    )
    .await?
    {
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
                symbol: entry
                    .token_metadata
                    .symbol
                    .unwrap_or(if entry.token_address.is_none() {
                        network.symbol.unwrap_or(format!("{}ETH", network.name))
                    } else {
                        "UNKNOWN".to_string()
                    }),
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
            value: entry.token_balance,
        };

        if asset.usd_value().map(|v| v > 0.0).unwrap_or_default()
        // || has_token(&networks, &asset.r#type.token_address)
        {
            balances.push(asset);
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

    // Native balances are also fetched through Alchemy above
    // for network in networks.get_iter(config.testnet_mode) {
    //     let provider = network.get_provider()?;

    //     let balance = provider.get_balance(wallet_address).await?;
    //     if !balance.is_zero() {
    //         let price = if let Some(price_ticker) = &network.price_ticker {
    //             if price_ticker == "ETH" {
    //                 Price::InETH(1f64)
    //             } else {
    //                 let (price, _) = Alchemy::get_price(price_ticker).await.expect("api failed");
    //                 Price::InUSD(price)
    //             }
    //         } else {
    //             Price::Unknown
    //         };

    //         balances.push(Asset {
    //             wallet_address,
    //             r#type: AssetType {
    //                 token_address: TokenAddress::Native,
    //                 network: network.name.clone(),
    //                 symbol: network.symbol.clone().unwrap_or("ETH".to_string()),
    //                 name: network.name.clone(),
    //                 decimals: 18,
    //                 price,
    //             },
    //             value: balance,
    //         });
    //     }
    // }

    balances.sort_by(|a, b| {
        a.usd_value()
            .partial_cmp(&b.usd_value())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(balances)
}
