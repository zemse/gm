use std::{collections::HashMap, fmt::Display};

use alloy::{primitives::Address, providers::ProviderBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    disk::{Config, DiskInterface, FileFormat},
    utils::Provider,
};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Network {
    pub name: String,
    pub name_alchemy: Option<String>,
    pub chain_id: u32,
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
    pub contract_address: Address,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (chain_id: {})", self.name, self.chain_id)
    }
}

impl Network {
    pub fn get_rpc(&self) -> String {
        if let Some(rpc_url) = &self.rpc_url {
            rpc_url.clone()
        } else if let Some(rpc_alchemy) = &self.rpc_alchemy {
            rpc_alchemy.replace(
                "{}",
                // TODO handle this error when alchemy API key not present
                &Config::alchemy_api_key(),
            )
        } else if let Some(rpc_infura) = &self.rpc_infura {
            rpc_infura.clone()
        } else {
            // TODO remove this panic and allow user to gracefully handle this situation like providing
            // their own RPC URL or ALCHEMY_API_KEY
            panic!("No RPC URL found for network {}", self.name);
        }
    }

    pub fn get_tx_url(&self, tx_hash: &str) -> Option<String> {
        self.explorer_url
            .as_ref()
            .map(|explorer_url| explorer_url.replace("{}", tx_hash))
    }

    pub fn get_provider(&self) -> Provider {
        let rpc_url = self.get_rpc().parse().expect("error parsing URL");
        ProviderBuilder::new().on_http(rpc_url)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NetworkStore {
    pub networks: Vec<Network>,
}

impl DiskInterface for NetworkStore {
    const FILE_NAME: &'static str = "networks";
    const FORMAT: FileFormat = FileFormat::YAML;

    fn load() -> Self {
        let mut networks = HashMap::<u32, Network>::new();
        let mut insert = |entry: Network| {
            let existing = networks.remove(&entry.chain_id);
            let entry = if let Some(existing) = existing {
                // merge entries
                Network {
                    name: entry.name,
                    name_alchemy: entry.name_alchemy.or(existing.name_alchemy),
                    chain_id: entry.chain_id,
                    rpc_url: entry.rpc_url.or(existing.rpc_url),
                    rpc_alchemy: entry.rpc_alchemy.or(existing.rpc_alchemy),
                    rpc_infura: entry.rpc_infura.or(existing.rpc_infura),
                    explorer_url: entry.explorer_url.or(existing.explorer_url),
                    is_testnet: entry.is_testnet,
                    // TODO avoid duplicates
                    tokens: entry
                        .tokens
                        .iter()
                        .chain(existing.tokens.iter())
                        .cloned()
                        .collect(),
                }
            } else {
                entry
            };

            networks.insert(entry.chain_id, entry);
        };

        // TODO move this to a separate file

        insert(Network {
            name: "Mainnet".to_string(),
            name_alchemy: Some("eth-mainnet".to_string()),
            chain_id: 1,
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
        });

        insert(Network {
            name: "Arbitrum".to_string(),
            name_alchemy: Some("arb-mainnet".to_string()),
            chain_id: 42161,
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
        });

        insert(Network {
            name: "Base".to_string(),
            name_alchemy: Some("base-mainnet".to_string()),
            chain_id: 8453,
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
        });

        insert(Network {
            name: "Sepolia".to_string(),
            name_alchemy: Some("eth-sepolia".to_string()),
            chain_id: 11155111,
            rpc_url: None,
            rpc_alchemy: Some(("https://eth-sepolia.g.alchemy.com/v2/{}").to_string()),
            rpc_infura: None,
            explorer_url: Some("https://sepolia.etherscan.io/tx/{}".to_string()),
            is_testnet: true,
            tokens: vec![],
        });

        // load networks from disk and override defaults
        // TODO too many .clone() used here, improve it
        let store = NetworkStore::load_internal();
        for network in &store.networks {
            insert(network.clone());
        }
        store.save();

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
        store.save();
        store
    }
}

impl NetworkStore {
    pub fn get_by_name(&self, name: &str) -> Option<Network> {
        self.networks.iter().find(|n| n.name == name).cloned()
    }
}

impl From<String> for Network {
    fn from(value: String) -> Self {
        let networks = NetworkStore::load();
        networks.get_by_name(&value).expect("network not found")
    }
}
