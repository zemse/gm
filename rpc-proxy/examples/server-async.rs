use std::sync::mpsc;

use gm_rpc_proxy::rpc_types::ResponsePayload;
use serde_json::Value;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

struct ExternalMessage {
    content: String,
    reply_to: oneshot::Sender<ResponsePayload<Value>>,
}

#[tokio::main]
async fn main() {
    let (tr, rv) = mpsc::channel::<ExternalMessage>();
    let sd = CancellationToken::new();

    tokio::spawn(async move {
        gm_rpc_proxy::serve(
            3000,
            &"abcd",
            "http://127.0.0.1:8545".parse().unwrap(),
            sd,
            move |req| {
                if req.method == "eth_blockNumber" {
                    // Channel created for every async request
                    let (oneshot_tr, oneshot_rv) = oneshot::channel::<ResponsePayload<Value>>();

                    let _ = tr.send(ExternalMessage {
                        content: "custom_method is called, what to do now?".to_string(),
                        reply_to: oneshot_tr,
                    });

                    // Async response, server will wait for a message on oneshot_rv
                    Ok(gm_rpc_proxy::OverrideResult::Async(oneshot_rv))
                } else {
                    // This will cause the request to be forwarded to underlying rpc
                    Ok(gm_rpc_proxy::OverrideResult::NoOverride)
                }
            },
        )
        .await
        .unwrap();
    });

    let msg = rv.recv().unwrap();
    println!("Received external message: {}", msg.content);

    // Store the reply handle somewhere to use it later
    let mut reply_handle_store = Some(msg.reply_to);

    // Once response is prepared, use the handle to send the response back.
    // This response will be sent to the original caller of the RPC method
    reply_handle_store
        .take()
        .unwrap()
        .send(ResponsePayload::Success(Value::String("0x110".to_string())))
        .unwrap();
}
