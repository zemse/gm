use alloy::primitives::{Address, U256};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{disk::Config, error::Error}; // for building the JSON body

#[derive(Serialize, Deserialize, Debug)]
pub struct TokensByWalletEntry {
    pub address: Address,
    pub network: String,
    #[serde(rename = "tokenAddress")]
    pub token_address: Address,
    #[serde(rename = "tokenBalance")]
    pub token_balance: U256,
    #[serde(rename = "tokenMetadata")]
    pub token_metadata: TokenMetadata,
    #[serde(rename = "tokenPrices")]
    pub token_prices: Vec<TokenPricesEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenMetadata {
    pub symbol: String,
    pub decimals: u8,
    pub name: String,
    pub logo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenPricesEntry {
    pub currency: String,
    pub value: String,
    #[serde(rename = "lastUpdatedAt")]
    pub last_updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenBalancesByWalletEntry {
    address: Address,
    network: String,
    #[serde(rename = "tokenAddress")]
    token_address: Address,
    #[serde(rename = "tokenBalance")]
    token_balance: U256,
}

pub struct Alchemy;

impl Alchemy {
    pub fn api_key() -> String {
        Config::alchemy_api_key()
    }

    pub async fn get_price(symbol: &str) -> Result<(f64, String), Error> {
        let api_key = Self::api_key();
        let client = reqwest::Client::new();

        let res = client
            .get(format!(
                "https://api.g.alchemy.com/prices/v1/{api_key}/tokens/by-symbol?symbols={symbol}"
            ))
            .header("accept", "application/json")
            .send()
            .await?
            .json::<Value>()
            .await?;
        let res = res.as_object().ok_or("response not an object")?;

        let data = res.get("data").ok_or("data not found in response")?;

        let data = data
            .as_array()
            .ok_or("data not an object")?
            .first()
            .ok_or("data array is empty")?;

        let data_symbol = data
            .get("symbol")
            .ok_or("symbol not found in response")?
            .as_str()
            .ok_or("symbol not a string")?;

        if data_symbol != symbol {
            return Err("symbol in response does not match requested symbol".into());
        }

        let prices = data
            .get("prices")
            .ok_or("prices not found in response")?
            .as_array()
            .ok_or("prices not array")?
            .first()
            .ok_or("prices array is empty")?
            .as_object()
            .ok_or("prices[0] is not object")?;

        let currency = prices
            .get("currency")
            .ok_or("currency not found in prices[0]")?
            .as_str()
            .ok_or("currency not a string")?;

        if currency != "usd" {
            return Err("currency is not USD".into());
        }

        let value = prices
            .get("value")
            .ok_or("value not found in prices[0]")?
            .as_str()
            .ok_or("currency not a string")?;

        let last_updated_at = prices
            .get("lastUpdatedAt")
            .ok_or("lastUpdatedAt not found in prices[0]")?
            .as_str()
            .ok_or("currency not a string")?;

        Ok((value.parse()?, last_updated_at.to_string()))
    }

    // docs: https://docs.alchemy.com/reference/get-tokens-by-address
    pub async fn get_tokens_by_wallet(
        address: Address,
        networks: Vec<String>,
    ) -> Result<Vec<TokensByWalletEntry>, Error> {
        let api_key = Self::api_key();

        let mut result = Vec::new();
        for networks in networks.chunks(5) {
            // Build the request body using serde_json::json! macro:
            let body = json!({
                "addresses": [
                    {
                        "address": address,
                        "networks": networks
                    }
                ],
                "withMetadata": true,
                "withPrices": true
            });

            // Initialize the reqwest Client
            let client = Client::new();

            // Make the POST request
            let response = client
                .post(format!(
                    "https://api.g.alchemy.com/data/v1/{api_key}/assets/tokens/by-address"
                ))
                .header("accept", "application/json")
                .header("content-type", "application/json")
                .json(&body) // send JSON body
                .send() // execute the request
                .await? // await the response
                .json::<Value>() // Parse the JSON into serde_json::Value
                .await?;

            let response = response
                .get("data")
                .ok_or(Error::InternalError(format!(
                    "'data' not present in response {response:?}"
                )))?
                .get("tokens")
                .ok_or(Error::InternalError(format!(
                    "'tokens' not present in response {response:?}"
                )))?;

            let parsed: Vec<TokensByWalletEntry> = serde_json::from_value(response.clone())?;
            result.extend(parsed);
        }

        Ok(result)
    }

    pub async fn get_token_balances_by_wallet(
        address: Address,
    ) -> Result<Vec<TokenBalancesByWalletEntry>, Error> {
        // Build the request body using serde_json::json! macro:
        let body = json!({
            "addresses": [
                {
                    "address": address,
                    "networks": ["eth-mainnet", "base-mainnet", "matic-mainnet"]
                }
            ]
        });

        // Initialize the reqwest Client
        let client = Client::new();

        let api_key = Self::api_key();

        // Make the POST request
        let response = client
            .post(format!(
                "https://api.g.alchemy.com/data/v1/{api_key}/assets/tokens/balances/by-address"
            ))
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .json(&body) // send JSON body
            .send() // execute the request
            .await? // await the response
            .json::<Value>() // Parse the JSON into serde_json::Value
            .await?;

        let response = response
            .get("data")
            .expect("'data' not present in response")
            .get("tokens")
            .expect("'tokens' not present in response");

        let parsed: Vec<TokenBalancesByWalletEntry> = serde_json::from_value(response.clone())?;
        Ok(parsed)
    }
}
