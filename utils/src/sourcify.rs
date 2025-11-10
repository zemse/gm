use std::collections::HashMap;

use alloy::{
    hex,
    json_abi::AbiItem,
    primitives::{Address, FixedBytes},
};
use serde::Deserialize;

use crate::Reqwest;

// TODO enable customising the base URL via config
pub struct Sourcify;

#[derive(Debug, Deserialize)]
pub struct ContractsResponse<'a> {
    #[serde(default)]
    compilation: Option<Compilation>,
    #[allow(dead_code)]
    abi: AbiItem<'a>,
}

#[derive(Debug, Deserialize)]
struct Compilation {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignatureResponse {
    pub ok: bool,
    pub result: ResultBody,
}

#[derive(Debug, Deserialize)]
pub struct ResultBody {
    pub function: HashMap<String, Vec<FunctionEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct FunctionEntry {
    pub name: String,
    pub filtered: bool,
    #[serde(rename = "hasVerifiedContract")]
    pub has_verified_contract: bool,
}

#[derive(Debug, Deserialize)]
pub struct EventEntry {
    pub name: String,
    pub filtered: bool,
    #[serde(rename = "hasVerifiedContract")]
    pub has_verified_contract: bool,
}

impl Sourcify {
    pub async fn fetch_contract<'a>(
        chain_id: u64,
        contract_address: Address,
    ) -> crate::Result<ContractsResponse<'a>> {
        let url = format!(
            "https://repo.sourcify.dev/contracts/full_match/{}/{}.json",
            chain_id, contract_address
        );
        let resp = Reqwest::get(&url)?
            .query(&("fields", "abi,compilation"))
            .receive_json::<ContractsResponse>()
            .await?;
        Ok(resp)
    }

    pub async fn fetch_contract_name(
        chain_id: u64,
        contract_address: Address,
    ) -> crate::Result<Option<String>> {
        let resp = Self::fetch_contract(chain_id, contract_address).await?;
        Ok(resp.compilation.and_then(|comp| comp.name))
    }

    pub async fn lookup_signature(
        selector: FixedBytes<4>,
    ) -> crate::Result<Option<Vec<FunctionEntry>>> {
        let url = format!(
            "https://api.4byte.sourcify.dev/signature-database/v1/lookup?function={}&filter=true",
            hex::encode_prefixed(selector)
        );
        let mut resp = Reqwest::get(&url)?
            .query(&("fields", "compilation"))
            .receive_json::<SignatureResponse>()
            .await?;

        if resp.ok {
            return Ok(resp.result.function.remove(&hex::encode_prefixed(selector)));
        }

        Ok(None)
    }
}
