pub type Result<T> = std::result::Result<T, RpcProxyError>;

#[derive(Debug, thiserror::Error)]
pub enum RpcProxyError {
    #[error("Request parsing failed. (Error: {0})")]
    RequestParseFailed(serde_json::Error),

    #[error("Response formatting failed. (Error: {0})")]
    ResponseFormattingFailed(serde_json::Error),

    #[error("Failed to bind to port {0}. (Error: {1})")]
    PortBindingFailed(usize, std::io::Error),

    #[error("Server crashed. (Error: {0})")]
    ServerCrashed(std::io::Error),

    #[error("Oneshot channel receive failed. (Error: {0})")]
    OneshotRecvFailed(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("Timeout. (Error: {0})")]
    OneshotRecvTimeout(#[from] tokio::time::error::Elapsed),

    #[error("Forwarded RPC call failed. (Error: {0})")]
    ForwardedRequestFailed(#[from] reqwest::Error),
}
