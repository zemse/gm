use alloy::primitives::Address;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::utils::SerdeResponseParse;

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockScoutNetwork {
    #[serde(rename = "eth")]
    Mainnet,

    Arbitrum,
}

impl BlockScoutNetwork {
    fn api_base_url(&self) -> String {
        format!(
            "https://{}.blockscout.com",
            serde_plain::to_string(&self).unwrap()
        )
    }
}

pub struct BlockScout {
    pub network: BlockScoutNetwork,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    pub block_number_balance_updated_at: Option<u64>,
    pub coin_balance: Option<String>,
    pub creation_transaction_hash: Option<String>,
    pub creator_address_hash: Option<String>,
    pub ens_domain_name: Option<String>,
    pub exchange_rate: Option<String>,
    pub has_beacon_chain_withdrawals: Option<bool>,
    pub has_logs: Option<bool>,
    pub has_token_transfers: Option<bool>,
    pub has_tokens: Option<bool>,
    pub has_validated_blocks: Option<bool>,
    pub hash: Option<String>,
    // pub implementations: Option<Vec<String>>,
    pub is_contract: Option<bool>,
    pub is_scam: Option<bool>,
    pub is_verified: Option<bool>,
    pub metadata: Option<String>,
    pub name: Option<String>,
    pub private_tags: Option<Vec<String>>,
    pub proxy_type: Option<String>,
    pub public_tags: Option<Vec<String>>,
    pub token: Option<String>,
    pub watchlist_address_id: Option<String>,
    pub watchlist_names: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionMetadataTag {
    pub meta: serde_json::Value,
    pub name: String,
    pub ordinal: u64,
    pub slug: String,
    #[serde(rename = "tagType")]
    pub tag_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionMetadata {
    pub tags: Vec<TransactionMetadataTag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressInfo {
    pub ens_domain_name: Option<String>,
    pub hash: String,
    pub implementations: Option<serde_json::value::Value>,
    pub is_contract: bool,
    pub is_scam: bool,
    pub is_verified: bool,
    pub metadata: Option<TransactionMetadata>,
    pub name: Option<String>,
    pub private_tags: Vec<String>,
    pub proxy_type: Option<String>,
    pub public_tags: Vec<String>,
    pub watchlist_names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fee {
    #[serde(rename = "type")]
    pub fee_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub priority_fee: Option<String>,
    pub raw_input: String,
    pub result: String,
    pub hash: String,
    pub max_fee_per_gas: Option<String>,
    pub revert_reason: Option<serde_json::value::Value>,
    pub confirmation_duration: [f64; 2],
    pub transaction_burnt_fee: Option<String>,
    #[serde(rename = "type")]
    pub tx_type: u8,
    pub token_transfers_overflow: Option<String>,
    pub confirmations: u64,
    pub position: u64,
    pub max_priority_fee_per_gas: Option<String>,
    pub transaction_tag: Option<String>,
    pub created_contract: Option<String>,
    pub value: String,
    pub from: AddressInfo,
    pub gas_used: String,
    pub status: String,
    pub to: AddressInfo,
    pub authorization_list: Vec<String>,
    pub method: Option<String>,
    pub fee: Fee,
    pub actions: Vec<String>,
    pub gas_limit: String,
    pub gas_price: String,
    pub decoded_input: Option<serde_json::value::Value>,
    pub token_transfers: Option<String>,
    pub base_fee_per_gas: String,
    pub timestamp: String,
    pub nonce: u64,
    pub historic_exchange_rate: Option<String>,
    pub transaction_types: Vec<String>,
    pub exchange_rate: String,
    pub block_number: u64,
    pub has_error_in_internal_transactions: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub address_hash: String,
    pub circulating_market_cap: Option<String>,
    pub decimals: Option<String>,
    pub exchange_rate: Option<String>,
    pub holders: Option<String>,
    pub holders_count: String,
    pub icon_url: Option<String>,
    pub name: String,
    pub symbol: Option<String>,
    pub total_supply: String,
    #[serde(rename = "type")]
    pub token_type: String,
    pub volume_24h: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInstanceMetadata {
    pub description: String,
    pub image: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInstance {
    pub animation_url: Option<String>,
    pub external_app_url: Option<String>,
    pub id: String,
    pub image_url: String,
    pub is_unique: Option<bool>,
    pub media_type: Option<String>,
    pub media_url: String,
    pub metadata: TokenInstanceMetadata,
    pub owner: Option<String>,
    pub thumbnails: Option<String>,
    pub token: TokenInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Total {
    pub decimals: Option<String>,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenTransferRecord {
    pub block_hash: String,
    pub block_number: u64,
    pub from: AddressInfo,
    pub log_index: u64,
    pub method: String,
    pub timestamp: String,
    pub to: AddressInfo,
    pub token: TokenInfo,
    pub total: Total,
    pub transaction_hash: String,
    #[serde(rename = "type")]
    pub transfer_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub address: String,
    pub address_hash: String,
    pub circulating_market_cap: Option<String>,
    pub decimals: Option<String>,
    pub exchange_rate: Option<String>,
    pub holders: Option<String>,
    pub holders_count: Option<String>,
    pub icon_url: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub total_supply: Option<String>,
    #[serde(rename = "type")]
    pub token_type: String,
    pub volume_24h: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenBalance {
    pub token: Token,
    pub token_id: Option<String>,
    pub token_instance: Option<serde_json::Value>, // It’s null, so keep it generic
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NextPageParams {
    pub block_number: u64,
    pub index: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockScoutResponse<T> {
    pub next_page_params: Option<NextPageParams>,
    pub items: Vec<T>,
}

impl BlockScout {
    async fn address_info(&self, address: Address) -> crate::Result<AccountInfo> {
        let url = format!(
            "{}/api/v2/addresses/{}",
            self.network.api_base_url(),
            address
        );

        println!("Fetching address info from: {}", url);

        let client = reqwest::Client::new();
        client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .serde_parse_custom()
            .await
    }

    pub async fn address_transactions(
        &self,
        address: Address,
    ) -> crate::Result<BlockScoutResponse<Transaction>> {
        let url = format!(
            "{}/api/v2/addresses/{}/transactions",
            self.network.api_base_url(),
            address
        );

        let client = reqwest::Client::new();
        client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .serde_parse_custom()
            .await
    }

    pub async fn token_transfers(
        &self,
        address: Address,
    ) -> crate::Result<BlockScoutResponse<TokenBalance>> {
        let url = format!(
            "{}/api/v2/addresses/{}/token-transfers",
            self.network.api_base_url(),
            address
        );

        let client = Client::new();
        client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .serde_parse_custom()
            .await
    }

    pub async fn token_balances(&self, address: Address) -> crate::Result<Vec<TokenBalance>> {
        let url = format!(
            "{}/api/v2/addresses/{}/token-balances",
            self.network.api_base_url(),
            address
        );

        let client = Client::new();
        let response = client.get(url).send().await?.error_for_status()?;

        serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(
            &response.text().await?,
        ))
        .map_err(crate::Error::SerdePathToError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_address_info() {
        let blockscout = BlockScout {
            network: BlockScoutNetwork::Mainnet,
        };

        let result = blockscout
            .address_info(
                "0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
                    .parse()
                    .unwrap(),
            )
            .await
            .expect("Failed to fetch address info");

        println!("{:#?}", result);

        // assert!(false);
    }

    #[tokio::test]
    async fn test_address_transactions() {
        let blockscout = BlockScout {
            network: BlockScoutNetwork::Mainnet,
        };

        let result = blockscout
            .address_transactions(
                "0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
                    .parse()
                    .unwrap(),
            )
            .await
            .expect("Failed to fetch address transactions");

        println!("{:#?}", result);

        // assert!(false);
    }

    #[tokio::test]
    async fn test_token_transfers() {
        let blockscout = BlockScout {
            network: BlockScoutNetwork::Mainnet,
        };

        let result = blockscout
            .token_transfers(
                "0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
                    .parse()
                    .unwrap(),
            )
            .await
            .expect("Failed to fetch token transfers");

        println!("{:#?}", result);

        // assert!(false);
    }

    #[tokio::test]
    async fn test_token_balances() {
        let blockscout = BlockScout {
            network: BlockScoutNetwork::Mainnet,
        };

        let result = blockscout
            .token_balances(
                "0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
                    .parse()
                    .unwrap(),
            )
            .await
            .expect("Failed to fetch token balances");

        println!("{:#?}", result);

        // assert!(false);
    }
}
