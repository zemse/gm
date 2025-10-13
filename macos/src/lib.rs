mod auth;
mod error;
mod keychain;

pub use error::MacosError as Error;
pub(crate) use error::Result;
pub use keychain::{
    get_account_list, get_secret, sign_message_async, store_mnemonic_wallet, store_private_key,
};
