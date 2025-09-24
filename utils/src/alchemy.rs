use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{config::Config, Error, Reqwest};

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub symbol: String,
    pub prices: Vec<Price>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Price {
    pub currency: String,
    pub value: String,
    #[serde(rename = "lastUpdatedAt")]
    pub last_updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokensByWalletEntry {
    pub address: Address,
    pub network: String,
    #[serde(rename = "tokenAddress")]
    pub token_address: Option<Address>,
    #[serde(rename = "tokenBalance")]
    pub token_balance: Option<U256>,
    #[serde(rename = "tokenMetadata")]
    pub token_metadata: TokenMetadata,
    #[serde(rename = "tokenPrices")]
    pub token_prices: Vec<TokenPricesEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenMetadata {
    pub symbol: Option<String>,
    #[serde(default)]
    pub decimals: Option<u8>,
    pub name: Option<String>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Tokens<T> {
    tokens: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlchemyData<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokensByWallet {
    pub tokens: Vec<TokensByWalletEntry>,
}

pub struct Alchemy;

impl Alchemy {
    // TODO move this into price manager
    pub async fn get_price(symbol: &str) -> crate::Result<(f64, String)> {
        let api_key = Config::alchemy_api_key(true)?;

        let response = Reqwest::get(format!(
            "https://api.g.alchemy.com/prices/v1/{api_key}/tokens/by-symbol?symbols={symbol}"
        ))?
        .receive_json::<AlchemyData<Vec<Asset>>>()
        .await?;

        let asset = response
            .data
            .into_iter()
            .find(|asset| asset.symbol == symbol)
            .ok_or_else(|| crate::Error::AlchemyResponse("asset not found"))?;

        let usd_price = asset
            .prices
            .into_iter()
            .find(|price| price.currency == "usd")
            .ok_or_else(|| crate::Error::AlchemyResponse("usd price not found"))?;

        Ok((usd_price.value.parse()?, usd_price.last_updated_at))
    }

    // docs: https://docs.alchemy.com/reference/get-tokens-by-address
    pub async fn get_tokens_by_wallet(
        address: Address,
        networks: Vec<String>,
    ) -> Result<Vec<TokensByWalletEntry>, Error> {
        let api_key = Config::alchemy_api_key(true)?;

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

            let url =
                format!("https://api.g.alchemy.com/data/v1/{api_key}/assets/tokens/by-address");
            let response = Reqwest::post(url)?
                .json_body(&body)
                .receive_json::<AlchemyData<TokensByWallet>>()
                .await?;

            result.extend(response.data.tokens);
        }

        Ok(result)
    }

    pub async fn get_token_balances_by_wallet(
        address: Address,
    ) -> Result<Vec<TokenBalancesByWalletEntry>, Error> {
        let body = json!({
            "addresses": [
                {
                    "address": address,
                    "networks": ["eth-mainnet", "base-mainnet", "matic-mainnet"]
                }
            ]
        });

        let api_key = Config::alchemy_api_key(true)?;

        let url = format!(
            "https://api.g.alchemy.com/data/v1/{api_key}/assets/tokens/balances/by-address"
        );

        let response = Reqwest::post(url)?
            .json_body(&body)
            .receive_json::<AlchemyData<Tokens<TokenBalancesByWalletEntry>>>()
            .await?;

        Ok(response.data.tokens)
    }
}
