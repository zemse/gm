use std::fmt::{Display, Formatter};

use alloy::primitives::{
    map::HashMap,
    utils::{format_units, parse_units},
    U256,
};
use fusion_plus_sdk::multichain_address::MultichainAddress;

use crate::{
    alchemy::Alchemy,
    disk::{Config, DiskInterface},
    network::{Network, NetworkStore, Token},
};

#[derive(Default)]
pub struct AssetManager {
    assets: HashMap<MultichainAddress, Option<Vec<Asset>>>,
    // prices: HashMap<String, Price>,
}

impl AssetManager {
    pub fn clear_data_for(&mut self, account: MultichainAddress) {
        self.assets.remove(&account);
    }

    // Update asset
    pub fn update_assets(
        &mut self,
        account: MultichainAddress,
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
        account: MultichainAddress,
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
    pub fn get_assets(&self, address: &MultichainAddress) -> Option<&Vec<Asset>> {
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
    Contract(MultichainAddress),
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
    pub chain_id: u32,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price: Price,
}

impl Display for AssetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.symbol, self.network)
    }
}

impl AssetType {
    pub fn parse_str(s: &str) -> crate::Result<(Token, Network)> {
        let start_paren = s.find('(').ok_or_else(|| {
            crate::Error::InternalError(format!("AssetType::parse_str({s:?}) .find '(' failed"))
        })?;
        let end_paren = s.find(')').ok_or_else(|| {
            crate::Error::InternalError(format!("AssetType::parse_str({s:?}) .find ')' failed"))
        })?;

        // Extract the symbol (trim to remove trailing space)
        let symbol = s[..start_paren].trim();

        // Extract the network name inside parentheses
        let network_name = s[start_paren + 1..end_paren].trim();
        let network = NetworkStore::from_name(network_name)?;

        let token = network
            .tokens
            .iter()
            .find(|token| token.symbol == symbol)
            .ok_or(crate::Error::InternalError(format!(
                "token {symbol:?} not found"
            )))?
            .clone();

        Ok((token, network))
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
    pub wallet_address: MultichainAddress,
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

#[allow(dead_code)]
fn has_token(networks: &NetworkStore, token_address: &TokenAddress) -> bool {
    match token_address {
        TokenAddress::Native => false,
        TokenAddress::Contract(address) => networks.has_token(address),
    }
}

pub async fn get_all_assets_mock() -> crate::Result<(MultichainAddress, Vec<Asset>)> {
    let config = Config::load()?;
    let wallet_address = config
        .current_account
        .ok_or(crate::Error::CurrentAccountNotSet)?;

    Ok((
        wallet_address,
        vec![
            Asset {
                wallet_address,
                r#type: AssetType {
                    token_address: TokenAddress::Native,
                    network: "Mainnet".to_string(),
                    chain_id: 1,
                    symbol: "ETH".to_string(),
                    name: "Ethereum".to_string(),
                    decimals: 18,
                    price: Price::InUSD(3500.0),
                },
                value: parse_units("1.2", 18).unwrap().get_absolute(),
                light_client_verification: LightClientVerification::Pending,
            },
            Asset {
                wallet_address,
                r#type: AssetType {
                    token_address: TokenAddress::Native,
                    network: "Tron".to_string(),
                    chain_id: 728126428,
                    symbol: "USDT".to_string(),
                    name: "USD Tether".to_string(),
                    decimals: 6,
                    price: Price::InUSD(1.0),
                },
                value: U256::from(1e10),
                light_client_verification: LightClientVerification::Pending,
            },
            Asset {
                wallet_address,
                r#type: AssetType {
                    token_address: TokenAddress::Native,
                    network: "Mainnet".to_string(),
                    chain_id: 1,
                    symbol: "USDT".to_string(),
                    name: "USD Tether".to_string(),
                    decimals: 6,
                    price: Price::InUSD(1.0),
                },
                value: U256::from(2e10),
                light_client_verification: LightClientVerification::Pending,
            },
        ],
    ))
}

pub async fn get_all_assets() -> crate::Result<(MultichainAddress, Vec<Asset>)> {
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
                chain_id: network.chain_id,
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
            light_client_verification: LightClientVerification::Pending,
        };

        if asset.value > U256::ZERO
            && (config.testnet_mode || asset.usd_value().map(|v| v > 0.0).unwrap_or_default())
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

    Ok((wallet_address, balances))
}
