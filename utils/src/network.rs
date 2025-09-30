use std::{collections::HashMap, fmt::Display};

use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
};
use serde::{Deserialize, Serialize};
use serde_with::{formats::PreferMany, serde_as, skip_serializing_none, OneOrMany};

use crate::{
    config::Config,
    disk_storage::{DiskStorageInterface, FileFormat},
};

#[skip_serializing_none]
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Network {
    pub name: String,
    pub name_alchemy: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde_as(as = "OneOrMany<_, PreferMany>")]
    pub name_aliases: Vec<String>,
    pub chain_id: u32,
    pub symbol: Option<String>,
    pub native_decimals: Option<u8>,
    pub price_ticker: Option<String>,
    pub rpc_url: Option<String>,
    pub rpc_alchemy: Option<String>,
    pub rpc_infura: Option<String>,
    pub explorer_url: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_testnet: bool,
    pub rpc_port: Option<usize>,
    pub chainlink_native_price_feed: Option<Address>,
    pub chainlink_native_price_feed_decimals: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<Token>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub contract_address: Address,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (chain_id: {})", self.name, self.chain_id)
    }
}

impl Network {
    pub fn from_name(network_name: &str) -> crate::Result<Network> {
        let network_store = NetworkStore::load()?;
        network_store
            .get_by_name(network_name)
            .ok_or(crate::Error::NetworkNotFound(network_name.to_string()))
    }

    pub fn from_chain_id(chain_id: u32) -> crate::Result<Network> {
        let network_store = NetworkStore::load()?;
        network_store
            .get_by_chain_id(chain_id)
            .ok_or(crate::Error::NetworkNotFound(format!(
                "Chain ID {chain_id}",
            )))
    }

    // TODO make this return type as Url
    pub fn get_rpc(&self) -> crate::Result<String> {
        if let Some(rpc_url) = &self.rpc_url {
            Ok(rpc_url.clone())
        } else if let Some(rpc_alchemy) = &self.rpc_alchemy {
            Ok(rpc_alchemy.replace("{}", &Config::alchemy_api_key(true)?))
        } else if let Some(name_alchemy) = &self.name_alchemy {
            Ok(format!(
                "https://{}.g.alchemy.com/v2/{}",
                name_alchemy,
                Config::alchemy_api_key(true)?
            ))
        } else if let Some(rpc_infura) = &self.rpc_infura {
            Ok(rpc_infura.clone())
        } else {
            Err(crate::Error::RpcUrlNotFound {
                network: self.name.clone(),
                chain_id: self.chain_id,
            })
        }
    }

    pub fn get_tx_url(&self, tx_hash: &str) -> Option<String> {
        self.explorer_url
            .as_ref()
            .map(|explorer_url| explorer_url.replace("{}", tx_hash))
    }

    pub fn get_provider(&self) -> crate::Result<impl Provider> {
        let rpc_url = self.get_rpc()?;

        rpc_url
            .parse()
            .map_err(|e| crate::Error::UrlParsingFailed(rpc_url, e))
            .map(|rpc_url| ProviderBuilder::new().connect_http(rpc_url))
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NetworkStore {
    pub networks: Vec<Network>,
}

impl DiskStorageInterface for NetworkStore {
    const FILE_NAME: &'static str = "networks";
    const FORMAT: FileFormat = FileFormat::YAML;
}

impl NetworkStore {
    pub fn filter(self, testnet_mode: bool) -> Vec<Network> {
        self.networks
            .into_iter()
            .filter(|n| n.is_testnet == testnet_mode)
            .collect()
    }

    pub fn load_and_update() -> crate::Result<Self> {
        let mut networks = HashMap::<u32, Network>::new();

        let merge_tokens = |a: Vec<Token>, b: Vec<Token>| {
            let mut tokens = {
                let mut map = HashMap::<Address, Token>::new();
                for token in a.into_iter().chain(b) {
                    map.insert(token.contract_address, token);
                }
                map.values().cloned().collect::<Vec<Token>>()
            };
            tokens.sort_by(|a, b| a.contract_address.cmp(&b.contract_address));
            tokens
        };

        let mut insert = |new_entry: Network| {
            let existing = networks.remove(&new_entry.chain_id);
            // merge props from entry into existing
            let entry = if let Some(existing) = existing {
                let mut name_aliases = vec![];
                for n in new_entry
                    .name_aliases
                    .iter()
                    .chain(existing.name_aliases.iter())
                {
                    if !name_aliases.contains(n) {
                        name_aliases.push(n.clone());
                    }
                }
                Network {
                    name: new_entry.name,
                    name_alchemy: new_entry.name_alchemy.or(existing.name_alchemy),
                    name_aliases,
                    chain_id: new_entry.chain_id,
                    symbol: new_entry.symbol.or(existing.symbol),
                    native_decimals: new_entry.native_decimals.or(existing.native_decimals),
                    price_ticker: new_entry.price_ticker.or(existing.price_ticker),
                    rpc_url: new_entry.rpc_url.or(existing.rpc_url),
                    rpc_alchemy: new_entry.rpc_alchemy.or(existing.rpc_alchemy),
                    rpc_infura: new_entry.rpc_infura.or(existing.rpc_infura),
                    explorer_url: new_entry.explorer_url.or(existing.explorer_url),
                    is_testnet: new_entry.is_testnet,
                    rpc_port: new_entry.rpc_port.or(existing.rpc_port),
                    chainlink_native_price_feed: new_entry
                        .chainlink_native_price_feed
                        .or(existing.chainlink_native_price_feed),
                    chainlink_native_price_feed_decimals: new_entry
                        .chainlink_native_price_feed_decimals
                        .or(existing.chainlink_native_price_feed_decimals),
                    tokens: merge_tokens(new_entry.tokens, existing.tokens),
                }
            } else {
                new_entry
            };

            networks.insert(entry.chain_id, entry);
        };

        // First place the default networks
        for network in default_networks() {
            insert(network);
        }

        // Then place the user defined networks to override the defaults
        let store = NetworkStore::load()?;
        for network in store.networks {
            insert(network);
        }

        // Sort by chain ID and keep testnets at the bottom
        let mut networks: Vec<Network> = networks.values().cloned().collect();
        networks.sort_by(|a, b| {
            a.chain_id
                .cmp(&b.chain_id)
                .then(a.is_testnet.cmp(&b.is_testnet))
        });

        let store_updated = NetworkStore { networks };
        store_updated.save()?;
        Ok(store_updated)
    }

    pub fn get_by_name(&self, network_name: &str) -> Option<Network> {
        self.networks
            .iter()
            .find(|n| {
                n.name == network_name
                    || n.name_alchemy
                        .as_ref()
                        .map(|name| name == network_name)
                        .unwrap_or(false)
                    || n.name_aliases.contains(&network_name.to_string())
            })
            .cloned()
    }

    pub fn get_by_chain_id(&self, chain_id: u32) -> Option<Network> {
        self.networks
            .iter()
            .find(|n| n.chain_id == chain_id)
            .cloned()
    }

    pub fn get_alchemy_network_names(&self, testnet_mode: bool) -> Vec<String> {
        self.networks
            .iter()
            .filter_map(|n| {
                (n.is_testnet == testnet_mode)
                    .then_some(n.name_alchemy.clone())
                    .flatten()
            })
            .collect()
    }

    pub fn get_iter(&self, testnet_mode: bool) -> impl Iterator<Item = &Network> {
        self.networks
            .iter()
            .filter(move |n| n.is_testnet == testnet_mode)
    }

    pub fn register_token(
        &mut self,
        network_name: &str,
        token_address: Address,
        token_symbol: Option<&str>,
        token_name: &str,
        token_decimals: u8,
    ) {
        let network = self
            .networks
            .iter_mut()
            .find(|n| {
                n.name == network_name
                    || n.name_alchemy
                        .as_ref()
                        .map(|name| name == network_name)
                        .unwrap_or(false)
            })
            .expect("network not found");

        let result = network
            .tokens
            .iter()
            .find(|token| token.contract_address == token_address);

        if result.is_none() {
            network.tokens.push(Token {
                name: token_name.to_string(),
                symbol: token_symbol.unwrap_or("UNKNOWN").to_string(),
                decimals: token_decimals,
                contract_address: token_address,
            });
        }
    }

    pub fn has_token(&self, token_address: &Address) -> bool {
        self.networks.iter().any(|network| {
            network
                .tokens
                .iter()
                .any(|token| token.contract_address == *token_address)
        })
    }
}

impl TryFrom<String> for Network {
    type Error = crate::Error;

    fn try_from(value: String) -> crate::Result<Self> {
        let networks = NetworkStore::load()?;
        networks
            .get_by_name(&value)
            .ok_or(crate::Error::NetworkNotFound(value))
    }
}

fn default_networks() -> Vec<Network> {
    vec![
        Network {
            name: "Mainnet".to_string(),
            name_alchemy: Some("eth-mainnet".to_string()),
            name_aliases: vec![],
            chain_id: 1,
            symbol: Some("ETH".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("ETH".to_string()),
            rpc_url: None,
            rpc_alchemy: Some(("https://eth-mainnet.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: None,
            is_testnet: false,
            rpc_port: None,
            chainlink_native_price_feed: Some(
                "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419"
                    .parse()
                    .unwrap(),
            ),
            chainlink_native_price_feed_decimals: Some(8),
            tokens: vec![
                Token {
                    name: "Wrapped Ether".to_string(),
                    symbol: "WETH".to_string(),
                    decimals: 18,
                    contract_address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
                        .parse()
                        .unwrap(),
                },
                Token {
                    name: "MakerDAO's DAI".to_string(),
                    symbol: "DAI".to_string(),
                    decimals: 18,
                    contract_address: "0x6b175474e89094c44da98b954eedeac495271d0f"
                        .parse()
                        .unwrap(),
                },
                Token {
                    name: "Coinbase USD Coin".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    contract_address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                        .parse()
                        .unwrap(),
                },
                Token {
                    name: "Tether USD".to_string(),
                    symbol: "USDT".to_string(),
                    decimals: 6,
                    contract_address: "0xdAC17F958D2ee523a2206206994597C13D831ec7"
                        .parse()
                        .unwrap(),
                },
            ],
        },
        Network {
            name: "Arbitrum".to_string(),
            name_alchemy: Some("arb-mainnet".to_string()),
            name_aliases: vec![],
            chain_id: 42161,
            symbol: Some("ArbETH".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("ETH".to_string()),
            rpc_url: None,
            rpc_alchemy: Some(("https://arb-mainnet.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: Some("https://arbiscan.io/tx/{}".to_string()),
            is_testnet: false,
            rpc_port: None,
            chainlink_native_price_feed: None,
            chainlink_native_price_feed_decimals: None,
            tokens: vec![
                Token {
                    name: "Wrapped Ether".to_string(),
                    symbol: "WETH".to_string(),
                    decimals: 18,
                    contract_address: "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"
                        .parse()
                        .unwrap(),
                },
                Token {
                    name: "MakerDAO's DAI".to_string(),
                    symbol: "DAI".to_string(),
                    decimals: 18,
                    contract_address: "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1"
                        .parse()
                        .unwrap(),
                },
                Token {
                    name: "Coinbase USD Coin".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    contract_address: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"
                        .parse()
                        .unwrap(),
                },
                Token {
                    name: "Coinbase USD Coin Bridged".to_string(),
                    symbol: "USDC(Bridged)".to_string(),
                    decimals: 6,
                    contract_address: "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8"
                        .parse()
                        .unwrap(),
                },
                Token {
                    name: "Tether USD".to_string(),
                    symbol: "USDT".to_string(),
                    decimals: 6,
                    contract_address: "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9"
                        .parse()
                        .unwrap(),
                },
            ],
        },
        Network {
            name: "Optimism".to_string(),
            name_alchemy: Some("opt-mainnet".to_string()),
            name_aliases: vec![],
            chain_id: 10,
            symbol: Some("OpETH".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("ETH".to_string()),
            rpc_url: None,
            rpc_alchemy: Some(("https://opt-mainnet.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: None,
            is_testnet: false,
            rpc_port: None,
            chainlink_native_price_feed: None,
            chainlink_native_price_feed_decimals: None,
            tokens: vec![],
        },
        Network {
            name: "Base".to_string(),
            name_alchemy: Some("base-mainnet".to_string()),
            name_aliases: vec![],
            chain_id: 8453,
            symbol: Some("BaseETH".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("ETH".to_string()),
            rpc_url: None,
            rpc_alchemy: Some(("https://base-mainnet.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: None,
            is_testnet: false,
            rpc_port: None,
            chainlink_native_price_feed: None,
            chainlink_native_price_feed_decimals: None,
            tokens: vec![
                Token {
                    name: "Wrapped Ether".to_string(),
                    symbol: "WETH".to_string(),
                    decimals: 18,
                    contract_address: "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"
                        .parse()
                        .unwrap(),
                },
                Token {
                    name: "Coinbase USD Coin".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    contract_address: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"
                        .parse()
                        .unwrap(),
                },
            ],
        },
        Network {
            name: "Polygon".to_string(),
            name_alchemy: Some("polygon-mainnet".to_string()),
            name_aliases: vec!["matic-mainnet".to_string()],
            chain_id: 137,
            symbol: Some("PolygonETH".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("ETH".to_string()),
            rpc_url: None,
            rpc_alchemy: Some(("https://polygon-mainnet.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: None,
            is_testnet: false,
            rpc_port: None,
            chainlink_native_price_feed: None,
            chainlink_native_price_feed_decimals: None,
            tokens: vec![],
        },
        Network {
            name: "Sepolia".to_string(),
            name_alchemy: Some("eth-sepolia".to_string()),
            name_aliases: vec![],
            chain_id: 11155111,
            symbol: Some("sepoliaETH".to_string()),
            native_decimals: Some(18),
            price_ticker: None,
            rpc_url: None,
            rpc_alchemy: Some(("https://eth-sepolia.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: Some("https://sepolia.etherscan.io/tx/{}".to_string()),
            is_testnet: true,
            rpc_port: None,
            chainlink_native_price_feed: None,
            chainlink_native_price_feed_decimals: None,
            tokens: vec![],
        },
    ]
}
