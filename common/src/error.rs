use alloy::signers::local::LocalSignerError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to build a signer using the mnemonic phrase. (Error: {0})")]
    MnemonicSignerBuildFailed(LocalSignerError),

    #[error("Failed to create a signer from the private key bytes. (Error: {0})")]
    PrivateKeySignerBuildFailed(alloy::signers::k256::ecdsa::Error),
}
