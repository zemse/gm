//! A simple JSON-RPC proxy server with ability to override specific methods
//! and forward rest to underlying RPC server.
//!
//! # Examples
//! See the `examples` folder for usage examples.
pub mod error;
pub mod rpc_types;
mod serve;

pub use error::{Result, RpcProxyError as Error};
pub use serve::*;
