use alloy::primitives::Address;

pub type Result<T> = std::result::Result<T, MacosError>;

#[derive(Debug, thiserror::Error)]
pub enum MacosError {
    #[error(transparent)]
    CommonError(#[from] gm_common::Error),

    #[error("Not able to find the account {0} in your keychain. (Error: {1:?})")]
    AccountNotFoundInKeychain(Address, security_framework::base::Error),

    #[error("Failed to store account {0} in your keychain. (Error: {1:?})")]
    StoringAccountInKeychainFailed(Address, security_framework::base::Error),

    #[error("Failed to parse string from keychain secret for account {0}. (Error: {1:?})")]
    ParsingStringFromKeychainSecretFailed(Address, std::string::FromUtf8Error),

    #[error("Not able to parse address for the keychain item {0}. (Error: {1:?})")]
    ParsingAddressFromKeychainFailed(String, alloy::hex::FromHexError),

    #[error("Private key for account {0} is invalid. (Error: {1:?})")]
    PrivateKeyInvalid(Address, alloy::signers::k256::ecdsa::Error),

    #[error("Please add a password or setup TouchID on your mac.")]
    AuthNotAvailable,

    #[error("Authentication failed because user denied permission.")]
    AuthFailed,

    #[error("Message signing failed. (Error: {0})")]
    MessageSigningFailed(alloy::signers::Error),
}
