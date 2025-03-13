use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::disk::DiskInterface;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Network {
    pub name: String,
    pub chain_id: u32,
    pub rpc_url: Option<String>, // TODO this can rather be an array
    pub rpc_alchemy: Option<String>,
    pub rpc_infura: Option<String>,
    pub explorer_url: Option<String>,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (chain_id: {})", self.name, self.chain_id)
    }
}

impl Network {
    pub fn get_rpc(&self) -> String {
        let config = crate::disk::Config::load();
        if let Some(rpc_url) = &self.rpc_url {
            rpc_url.clone()
        } else if let Some(rpc_alchemy) = &self.rpc_alchemy {
            rpc_alchemy.replace(
                "{}",
                config
                    .alchemy_api_key
                    .as_ref()
                    // TODO ask user to provide this
                    .expect("ALCHEMY_API_KEY is not net in config"),
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
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NetworkStore {
    pub networks: Vec<Network>,
}

impl DiskInterface for NetworkStore {
    const FILE_NAME: &'static str = "networks.toml";
}

pub fn get_networks() -> Vec<Network> {
    let mut networks = HashMap::new();
    let mut insert = |v: Network| {
        networks.insert(v.chain_id, v);
    };

    insert(Network {
        name: "mainnet".to_string(),
        chain_id: 1,
        rpc_url: None,
        rpc_alchemy: Some(("https://eth-mainnet.g.alchemy.com/v2/{}").to_string()),
        rpc_infura: None,
        explorer_url: None,
    });

    insert(Network {
        name: "sepolia".to_string(),
        chain_id: 11155111,
        rpc_url: None,
        rpc_alchemy: Some(("https://eth-sepolia.g.alchemy.com/v2/{}").to_string()),
        rpc_infura: None,
        explorer_url: Some("https://sepolia.etherscan.io/tx/{}".to_string()),
    });

    // load networks from disk and override defaults
    let store = NetworkStore::load();
    for network in store.networks {
        insert(network);
    }
    networks.values().cloned().collect()
}

impl From<String> for Network {
    fn from(value: String) -> Self {
        let networks = get_networks();
        let network = networks
            .iter()
            .find(|n| n.name == value)
            .expect("network not found");
        network.clone()
    }
}
