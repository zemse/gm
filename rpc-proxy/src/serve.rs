use axum::{extract::State, routing::post, Json, Router};
use reqwest::Client;
use serde_json::Value;
use std::{fmt, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::oneshot, time::timeout};
use url::Url;

use crate::rpc_types::{
    ErrorObj, Id, JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse, ResponsePayload, TwoPointZero,
};

/// The override closure should return this
pub enum OverrideResult {
    Sync(ResponsePayload<Value>),
    Async(oneshot::Receiver<ResponsePayload<Value>>),
    NoOverride,
}

#[derive(Clone)]
struct ServerState<F>
where
    F: Fn(JsonRpcRequest) -> crate::Result<OverrideResult> + Clone + Send + Sync + 'static,
{
    client: Client,
    fwd_to: Url,
    overrider: Arc<F>,
}

/// Start the RPC proxy server. This function will block the current thread during server lifetime.
/// It return an error if the server fails to start or crashes.
///
/// # Arguments
/// * `port` - Port to listen on.
/// * `secret` - Secret string to be included in the URL path for security.
/// * `fwd_to` - URL of the underlying RPC server to forward any un-overriden requests to.
/// * `overrider` - Closure to handle custom request overrides.
pub async fn serve<F>(
    port: usize,
    secret: &impl fmt::Display,
    fwd_to: Url,
    overrider: F,
) -> crate::Result<()>
where
    F: Fn(JsonRpcRequest) -> crate::Result<OverrideResult> + Clone + Send + Sync + 'static,
{
    let state = ServerState {
        client: Client::new(),
        fwd_to,
        overrider: Arc::new(overrider),
    };

    let app = Router::new()
        .route(&format!("/{secret}"), post(handler))
        .with_state(state);
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .map_err(|e| crate::Error::PortBindingFailed(port, e))?;
    axum::serve(listener, app)
        .await
        .map_err(crate::Error::ServerCrashed)?;

    Ok(())
}

async fn handler<F>(State(state): State<ServerState<F>>, Json(payload): Json<Value>) -> Json<Value>
where
    F: Fn(JsonRpcRequest) -> crate::Result<OverrideResult> + Clone + Send + Sync + 'static,
{
    match handle_batch_or_one(&state, payload).await {
        Ok(resp) => resp,
        Err(e) => Json(
            JsonRpcResponse {
                jsonrpc: TwoPointZero,
                payload: ResponsePayload::<Value>::Error(ErrorObj {
                    code: JsonRpcErrorCode::InternalError.as_i32(),
                    message: format!("Internal Error: {e}"),
                    data: None,
                }),
                id: Id::Null,
            }
            .to_value()
            .expect("internal error"),
        ),
    }
}

async fn handle_batch_or_one<F>(
    state: &ServerState<F>,
    payload: Value,
) -> crate::Result<Json<Value>>
where
    F: Fn(JsonRpcRequest) -> crate::Result<OverrideResult> + Clone + Send + Sync + 'static,
{
    if payload.is_array() {
        let requests: Vec<JsonRpcRequest> =
            serde_json::from_value(payload).map_err(crate::Error::RequestParseFailed)?;
        let mut outs = Vec::with_capacity(requests.len());
        for req in requests {
            let response = handle_one::<F>(state, req).await?;
            outs.push(serde_json::to_value(response).unwrap());
        }
        Ok(Json(Value::Array(outs)))
    } else {
        let request: JsonRpcRequest =
            serde_json::from_value(payload).map_err(crate::Error::RequestParseFailed)?;
        let response = handle_one::<F>(state, request).await?;
        Ok(Json(
            serde_json::to_value(&response).map_err(crate::Error::ResponseFormattingFailed)?,
        ))
    }
}

async fn handle_one<F>(
    state: &ServerState<F>,
    req: JsonRpcRequest,
) -> crate::Result<JsonRpcResponse<Value>>
where
    F: Fn(JsonRpcRequest) -> crate::Result<OverrideResult> + Clone + Send + Sync + 'static,
{
    match (state.overrider)(req.clone())? {
        OverrideResult::Sync(payload) => Ok(JsonRpcResponse {
            jsonrpc: TwoPointZero,
            payload,
            id: req.id,
        }),
        OverrideResult::Async(rx) => {
            let payload = timeout(Duration::from_secs(180), rx).await??;
            Ok(JsonRpcResponse {
                jsonrpc: TwoPointZero,
                payload,
                id: req.id,
            })
        }
        OverrideResult::NoOverride => {
            Ok(rpc_call(&state.client, state.fwd_to.clone(), &req).await?)
        }
    }
}

async fn rpc_call(
    client: &Client,
    url: Url,
    req: &JsonRpcRequest,
) -> Result<JsonRpcResponse<Value>, reqwest::Error> {
    client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&req)
        .send()
        .await?
        .json::<JsonRpcResponse<Value>>()
        .await
}
