# macos

- Contains utilities for macos to authenticate and interact with the keychain. It is only compiled for macos targets.
- Provides intent based functions to sign message or send specific transactions so that the dependent crates do not get access to the private key.
- [Alloy](https://github.com/alloy-rs/alloy) is a critical dependency that is used for signing messages and transactions.
