use gm_rpc_proxy::rpc_types::ResponsePayload;
use serde_json::Value;

#[tokio::main]
async fn main() {
    gm_rpc_proxy::serve(
        3000,
        "".to_string(),
        "http://127.0.0.1:8545".parse().unwrap(),
        |req| {
            if req.method == "eth_blockNumber" {
                // Synchronous immidiate response
                gm_rpc_proxy::OverrideResult::Sync(ResponsePayload::Success(Value::String(
                    "0x1".to_string(),
                )))
            } else {
                // This will cause the request to be forwarded to underlying rpc
                gm_rpc_proxy::OverrideResult::NoOverride
            }
        },
    )
    .await
    .unwrap();
}
