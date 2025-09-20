use std::fmt::Debug;

use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    Number(u64),
    String(String),
    #[default]
    Null,
}

#[derive(Clone, Copy, Debug)]
pub struct TwoPointZero;

impl Serialize for TwoPointZero {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("2.0")
    }
}

impl<'de> Deserialize<'de> for TwoPointZero {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "2.0" {
            Ok(TwoPointZero)
        } else {
            Err(D::Error::invalid_value(Unexpected::Str(&s), &"\"2.0\""))
        }
    }
}

#[derive(Debug)]
#[repr(i32)]
#[allow(dead_code)]
pub enum JsonRpcErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    ServerError(i32),
}

impl JsonRpcErrorCode {
    pub fn as_i32(&self) -> i32 {
        match *self {
            JsonRpcErrorCode::ParseError => -32700,
            JsonRpcErrorCode::InvalidRequest => -32600,
            JsonRpcErrorCode::MethodNotFound => -32601,
            JsonRpcErrorCode::InvalidParams => -32602,
            JsonRpcErrorCode::InternalError => -32603,
            JsonRpcErrorCode::ServerError(c) => c,
        }
    }
}

impl From<JsonRpcErrorCode> for ErrorObj {
    fn from(code: JsonRpcErrorCode) -> Self {
        let message = match code {
            JsonRpcErrorCode::ParseError => "Parse error",
            JsonRpcErrorCode::InvalidRequest => "Invalid Request",
            JsonRpcErrorCode::MethodNotFound => "Method not found",
            JsonRpcErrorCode::InvalidParams => "Invalid params",
            JsonRpcErrorCode::InternalError => "Internal error",
            JsonRpcErrorCode::ServerError(_) => "Server error",
        }
        .to_string();

        ErrorObj {
            code: code.as_i32(),
            message,
            data: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorObj {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl ErrorObj {
    pub fn user_denied() -> Self {
        ErrorObj {
            code: -4001,
            message: "User rejected the request.".to_string(),
            data: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponsePayload<T> {
    #[serde(rename = "result")]
    Success(T),
    #[serde(rename = "error")]
    Error(ErrorObj),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: TwoPointZero,
    pub method: String,
    pub params: Option<Value>,
    pub id: Id,
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: TwoPointZero,
    #[serde(flatten)]
    pub payload: ResponsePayload<T>,
    pub id: Id,
}

impl JsonRpcRequest {
    pub fn create_success_response<T>(&self, v: T) -> JsonRpcResponse<T> {
        JsonRpcResponse {
            jsonrpc: TwoPointZero,
            payload: ResponsePayload::Success(v),
            id: self.id.clone(),
        }
    }

    pub fn internal_error(&self, err: impl Debug) -> JsonRpcResponse<Value> {
        JsonRpcResponse {
            jsonrpc: TwoPointZero,
            payload: ResponsePayload::Error(ErrorObj {
                code: JsonRpcErrorCode::InternalError.as_i32(),
                message: format!("Internal error - {err:?}"),
                data: None,
            }),
            id: self.id.clone(),
        }
    }
}

impl<T> JsonRpcResponse<T> {
    pub fn to_value(&self) -> Result<Value, serde_json::Error>
    where
        T: Serialize,
    {
        serde_json::to_value(self)
    }
}
