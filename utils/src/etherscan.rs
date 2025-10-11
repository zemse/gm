use alloy::{hex, primitives::Address, signers::Signature};
use serde::Deserialize;
use serde_json::json;

use crate::Reqwest;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PublishResponse {
    d: Option<PublishResultData>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct PublishResultData {
    success: Option<bool>,
    verify_result: Option<String>,
    save_result: Option<String>,
    is_duplicated_message: Option<bool>,
    verified_message_location: Option<String>,
}

pub async fn publish_signature_to_etherscan(
    address: Address,
    message: String,
    signature: Signature,
) -> crate::Result<String> {
    let request = json!(
        {
            "address": address,
            // Etherscan does not support erc2098 signatures, export to legacy
            "messageSignature": hex::encode_prefixed(signature.as_bytes()),
            "messageRaw": message,
            "saveOption":"1"
        }
    );

    let result =
        Reqwest::post("https://etherscan.io/verifiedSignatures.aspx/VerifyMessageSignature")?
            .json_body(&request)
            .receive_json::<PublishResponse>()
            .await?;

    let location = result
        .d
        .as_ref()
        .and_then(|d| d.verified_message_location.as_ref())
        .ok_or_else(|| crate::Error::EtherscanPublishURLNotFound(format!("{result:?}")));

    location.map(|location| format!("https://etherscan.io{}", location))
}
