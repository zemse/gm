use std::{collections::HashMap, fmt::Display, str::FromStr};

use alloy::providers::ProviderBuilder;
use fusion_plus_sdk::multichain_address::MultichainAddress;
use serde::{Deserialize, Serialize};

use crate::{
    disk::{Config, DiskInterface, FileFormat},
    utils::Provider,
};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Network {
    pub name: String,
    pub name_alchemy: Option<String>,
    #[serde(default)]
    pub name_aliases: Vec<String>,
    pub chain_id: u32,
    #[serde(default)]
    pub multichain_address_requires_chain_id: bool,
    pub symbol: Option<String>,
    pub native_decimals: Option<u8>,
    pub price_ticker: Option<String>,
    pub rpc_url: Option<String>, // TODO this can rather be an array
    pub rpc_alchemy: Option<String>,
    pub rpc_infura: Option<String>,
    pub explorer_url: Option<String>,
    pub is_testnet: bool,
    pub tokens: Vec<Token>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub contract_address: MultichainAddress,
    pub network_name: String,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (chain_id: {})", self.name, self.chain_id)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.symbol, self.network_name)
    }
}

impl Token {
    pub fn parse_str(s: &str) -> crate::Result<(Token, Network)> {
        let start_paren = s.find('(').ok_or_else(|| {
            crate::Error::InternalError(format!("Token::parse_str({s:?}) .find '(' failed"))
        })?;
        let end_paren = s.find(')').ok_or_else(|| {
            crate::Error::InternalError(format!("Token::parse_str({s:?}) .find ')' failed"))
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

impl Network {
    pub fn get_rpc(&self) -> crate::Result<String> {
        if let Some(rpc_url) = &self.rpc_url {
            Ok(rpc_url.clone())
        } else if let Some(rpc_alchemy) = &self.rpc_alchemy {
            Ok(rpc_alchemy.replace(
                "{}",
                // TODO handle this error when alchemy API key not present
                &Config::alchemy_api_key()?,
            ))
        } else if let Some(name_alchemy) = &self.name_alchemy {
            Ok(format!(
                "https://{}.g.alchemy.com/v2/{}",
                name_alchemy,
                Config::alchemy_api_key()?
            ))
        } else if let Some(rpc_infura) = &self.rpc_infura {
            Ok(rpc_infura.clone())
        } else {
            // TODO remove this panic and allow user to gracefully handle this situation like providing
            // their own RPC URL or ALCHEMY_API_KEY
            Err(crate::Error::InternalError(format!(
                "No RPC URL found for network {} - chain_id {}",
                self.name, self.chain_id
            )))
        }
    }

    pub fn get_tx_url(&self, tx_hash: &str) -> Option<String> {
        self.explorer_url
            .as_ref()
            .map(|explorer_url| explorer_url.replace("{}", tx_hash))
    }

    pub fn get_provider(&self) -> crate::Result<Provider> {
        let rpc_url = self.get_rpc()?.parse()?;
        Ok(ProviderBuilder::new().connect_http(rpc_url))
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NetworkStore {
    pub networks: Vec<Network>,
}

impl DiskInterface for NetworkStore {
    const FILE_NAME: &'static str = "networks-fusion";
    const FORMAT: FileFormat = FileFormat::YAML;
}

impl NetworkStore {
    // TODO This function should be on Network
    pub fn from_name(network_name: &str) -> crate::Result<Network> {
        let network_store = NetworkStore::load()?;
        network_store
            .get_by_name(network_name)
            .ok_or(crate::Error::NetworkNotFound(network_name.to_string()))
    }

    // TODO This function should be on Network
    pub fn from_chain_id(chain_id: u32) -> crate::Result<Network> {
        let network_store = NetworkStore::load()?;
        network_store
            .get_by_chain_id(chain_id)
            .ok_or(crate::Error::NetworkNotFound(format!(
                "Chain ID {chain_id}",
            )))
    }

    pub fn load_networks(testnet_mode: bool) -> crate::Result<Vec<Network>> {
        Ok(NetworkStore::load()?
            .networks
            .into_iter()
            .filter(|n| n.is_testnet == testnet_mode)
            .collect())
    }

    pub fn sort_config() -> crate::Result<Self> {
        let mut networks = HashMap::<u32, Network>::new();

        let merge_tokens = |a: Vec<Token>, b: Vec<Token>| {
            let mut tokens = HashMap::<MultichainAddress, Token>::new();
            for token in a.into_iter().chain(b) {
                tokens.insert(token.contract_address, token);
            }
            let mut tokens = tokens.values().cloned().collect::<Vec<Token>>();
            tokens.sort_by(|a, b| a.contract_address.cmp(&b.contract_address));
            tokens
        };

        let mut insert = |entry: Network| {
            let existing = networks.remove(&entry.chain_id);
            let entry = if let Some(existing) = existing {
                // merge entries
                let mut name_aliases = vec![];
                for n in entry
                    .name_aliases
                    .iter()
                    .chain(existing.name_aliases.iter())
                {
                    if !name_aliases.contains(n) {
                        name_aliases.push(n.clone());
                    }
                }
                Network {
                    name: entry.name,
                    name_alchemy: entry.name_alchemy.or(existing.name_alchemy),
                    name_aliases,
                    chain_id: entry.chain_id,
                    multichain_address_requires_chain_id: entry
                        .multichain_address_requires_chain_id
                        || existing.multichain_address_requires_chain_id,
                    symbol: entry.symbol.or(existing.symbol),
                    native_decimals: entry.native_decimals.or(existing.native_decimals),
                    price_ticker: entry.price_ticker.or(existing.price_ticker),
                    rpc_url: entry.rpc_url.or(existing.rpc_url),
                    rpc_alchemy: entry.rpc_alchemy.or(existing.rpc_alchemy),
                    rpc_infura: entry.rpc_infura.or(existing.rpc_infura),
                    explorer_url: entry.explorer_url.or(existing.explorer_url),
                    is_testnet: entry.is_testnet,
                    tokens: merge_tokens(entry.tokens, existing.tokens),
                }
            } else {
                entry
            };

            networks.insert(entry.chain_id, entry);
        };

        for network in default_networks() {
            insert(network);
        }

        // load networks from disk and override defaults
        // TODO too many .clone() used here, improve it
        let store = NetworkStore::load()?;
        for network in &store.networks {
            insert(network.clone());
        }

        // Sort by chain ID and keep testnets at the bottom
        let mut networks: Vec<Network> = networks.values().cloned().collect();
        networks.sort_by(|a, b| {
            a.chain_id
                .cmp(&b.chain_id)
                .then(a.is_testnet.cmp(&b.is_testnet))
        });

        let store = NetworkStore {
            networks: networks.clone(),
        };

        store.save()?;
        Ok(store)
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
        token_address: MultichainAddress,
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
                network_name: network.name.clone(),
            });
        }
    }

    pub fn has_token(&self, token_address: &MultichainAddress) -> bool {
        self.networks.iter().any(|network| {
            network
                .tokens
                .iter()
                .any(|token| token.contract_address == *token_address)
        })
    }
}

impl FromStr for Network {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let networks = NetworkStore::load()?;
        networks
            .get_by_name(s)
            .ok_or(crate::Error::NetworkNotFound(s.to_string()))
    }
}

fn default_networks() -> Vec<Network> {
    vec![
        Network {
            name: "Mainnet".to_string(),
            name_alchemy: Some("eth-mainnet".to_string()),
            name_aliases: vec![],
            chain_id: 1,
            multichain_address_requires_chain_id: false,
            symbol: Some("ETH".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("ETH".to_string()),
            rpc_url: None,
            rpc_alchemy: Some(("https://eth-mainnet.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: None,
            is_testnet: false,
            tokens: vec![
                Token {
                    name: "Wrapped Ether".to_string(),
                    symbol: "WETH".to_string(),
                    decimals: 18,
                    contract_address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
                        .parse()
                        .unwrap(),
                    network_name: "Mainnet".to_string(),
                },
                Token {
                    name: "MakerDAO's DAI".to_string(),
                    symbol: "DAI".to_string(),
                    decimals: 18,
                    contract_address: "0x6b175474e89094c44da98b954eedeac495271d0f"
                        .parse()
                        .unwrap(),
                    network_name: "Mainnet".to_string(),
                },
                Token {
                    name: "Coinbase USD Coin".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    contract_address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                        .parse()
                        .unwrap(),
                    network_name: "Mainnet".to_string(),
                },
                Token {
                    name: "Tether USD".to_string(),
                    symbol: "USDT".to_string(),
                    decimals: 6,
                    contract_address: "0xdAC17F958D2ee523a2206206994597C13D831ec7"
                        .parse()
                        .unwrap(),
                    network_name: "Mainnet".to_string(),
                },
            ],
        },
        Network {
            name: "Arbitrum".to_string(),
            name_alchemy: Some("arb-mainnet".to_string()),
            name_aliases: vec![],
            chain_id: 42161,
            multichain_address_requires_chain_id: false,
            symbol: Some("ArbETH".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("ETH".to_string()),
            rpc_url: None,
            rpc_alchemy: Some(("https://arb-mainnet.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: Some("https://arbiscan.io/tx/{}".to_string()),
            is_testnet: false,
            tokens: vec![
                Token {
                    name: "Wrapped Ether".to_string(),
                    symbol: "WETH".to_string(),
                    decimals: 18,
                    contract_address: "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"
                        .parse()
                        .unwrap(),
                    network_name: "Arbitrum".to_string(),
                },
                Token {
                    name: "MakerDAO's DAI".to_string(),
                    symbol: "DAI".to_string(),
                    decimals: 18,
                    contract_address: "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1"
                        .parse()
                        .unwrap(),
                    network_name: "Arbitrum".to_string(),
                },
                Token {
                    name: "Coinbase USD Coin".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    contract_address: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"
                        .parse()
                        .unwrap(),
                    network_name: "Arbitrum".to_string(),
                },
                Token {
                    name: "Coinbase USD Coin Bridged".to_string(),
                    symbol: "USDC(Bridged)".to_string(),
                    decimals: 6,
                    contract_address: "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8"
                        .parse()
                        .unwrap(),
                    network_name: "Arbitrum".to_string(),
                },
                Token {
                    name: "Tether USD".to_string(),
                    symbol: "USDT".to_string(),
                    decimals: 6,
                    contract_address: "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9"
                        .parse()
                        .unwrap(),
                    network_name: "Arbitrum".to_string(),
                },
            ],
        },
        Network {
            name: "Optimism".to_string(),
            name_alchemy: Some("opt-mainnet".to_string()),
            name_aliases: vec![],
            chain_id: 10,
            multichain_address_requires_chain_id: false,
            symbol: Some("OpETH".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("ETH".to_string()),
            rpc_url: None,
            rpc_alchemy: Some(("https://opt-mainnet.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: None,
            is_testnet: false,
            tokens: vec![],
        },
        Network {
            name: "Base".to_string(),
            name_alchemy: Some("base-mainnet".to_string()),
            name_aliases: vec![],
            chain_id: 8453,
            multichain_address_requires_chain_id: false,
            symbol: Some("BaseETH".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("ETH".to_string()),
            rpc_url: None,
            rpc_alchemy: Some(("https://base-mainnet.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: None,
            is_testnet: false,
            tokens: vec![
                Token {
                    name: "Wrapped Ether".to_string(),
                    symbol: "WETH".to_string(),
                    decimals: 18,
                    contract_address: "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"
                        .parse()
                        .unwrap(),
                    network_name: "Base".to_string(),
                },
                Token {
                    name: "Coinbase USD Coin".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    contract_address: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"
                        .parse()
                        .unwrap(),
                    network_name: "Base".to_string(),
                },
            ],
        },
        Network {
            name: "Polygon".to_string(),
            name_alchemy: Some("polygon-mainnet".to_string()),
            name_aliases: vec!["matic-mainnet".to_string()],
            chain_id: 137,
            multichain_address_requires_chain_id: false,
            symbol: Some("PolygonETH".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("ETH".to_string()),
            rpc_url: None,
            rpc_alchemy: Some(("https://polygon-mainnet.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: None,
            is_testnet: false,
            tokens: vec![],
        },
        Network {
            name: "Tron".to_string(),
            name_alchemy: None,
            name_aliases: vec!["tron-mainnet".to_string()],
            chain_id: 728126428,
            multichain_address_requires_chain_id: true,
            symbol: Some("TRX".to_string()),
            native_decimals: Some(18),
            price_ticker: Some("TRX".to_string()),
            rpc_url: None,
            rpc_alchemy: None,
            rpc_infura: None,
            explorer_url: None,
            is_testnet: false,
            tokens: vec![Token {
                name: "Tether USD".to_string(),
                symbol: "USDT".to_string(),
                decimals: 6,
                contract_address: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".parse().unwrap(),
                network_name: "Tron".to_string(),
            }],
        },
        Network {
            name: "Sepolia".to_string(),
            name_alchemy: Some("eth-sepolia".to_string()),
            name_aliases: vec![],
            chain_id: 11155111,
            multichain_address_requires_chain_id: false,
            symbol: Some("sepoliaETH".to_string()),
            native_decimals: Some(18),
            price_ticker: None,
            rpc_url: None,
            rpc_alchemy: Some(("https://eth-sepolia.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: Some("https://sepolia.etherscan.io/tx/{}".to_string()),
            is_testnet: true,
            tokens: vec![],
        },
    ]
}
