use alloy::{
    consensus::{SignableTransaction, Signed, TxEip1559},
    hex,
    network::TxSignerSync,
    primitives::Address,
    signers::{
        k256::{ecdsa::SigningKey, FieldBytes},
        Signature, Signer,
    },
};
use core_foundation::{
    base::{CFCopyDescription, CFGetTypeID, TCFType},
    data::CFData,
    date::CFDate,
    dictionary::CFDictionary,
    string::CFString,
};
use gm_common::{secret::Secret, text_truncate::truncate_with_count, tx_meta::TransactionMeta};
use security_framework::{
    item::{ItemClass, ItemSearchOptions, SearchResult},
    os::macos::keychain::SecKeychain,
};
use std::collections::HashMap;

use crate::auth;

/// Stores the given mnemonic phrase in the keychain associated with the given address.
pub fn store_mnemonic_wallet(phrase: &str, address: Address) -> crate::Result<()> {
    let mnemonic_service = format!("gm:mnemonic:{address}");

    keychain()
        .add_generic_password(&mnemonic_service, &address.to_string(), phrase.as_bytes())
        .map_err(|e| crate::Error::StoringAccountInKeychainFailed(address, e))?;

    Ok(())
}

/// Stores the given private key in the keychain associated with the given address.
pub fn store_private_key(private_key: &FieldBytes, address: Address) -> crate::Result<()> {
    let pk_service = format!("gm:{address}");

    keychain()
        .add_generic_password(
            &pk_service,
            &address.to_string(),
            hex::encode(private_key).as_bytes(),
        )
        .map_err(|e| crate::Error::StoringAccountInKeychainFailed(address, e))?;

    Ok(())
}

/// Returns the list of all addresses stored in the keychain.
pub fn get_account_list() -> crate::Result<Vec<Address>> {
    let mut search = ItemSearchOptions::default();
    search.class(ItemClass::generic_password());
    // TODO configure this as this search misses some keys if user has more keychain items.
    search.limit(1000);
    search.load_attributes(true);

    let mut accounts = vec![];

    if let Ok(result) = search.search() {
        for item in result {
            if let SearchResult::Dict(item) = item {
                let item = simplify_dict(&item);
                let service = item.get("svce");
                if let Some(service) = service {
                    if service.starts_with("gm") {
                        let addr_str = item.get("acct").expect("must have an account address");
                        accounts.push(addr_str.parse().map_err(|e| {
                            crate::Error::ParsingAddressFromKeychainFailed(addr_str.clone(), e)
                        })?);
                    }
                }
            }
        }
    }

    Ok(accounts)
}

/// Opens an MacOS authentication prompt and on success, it returns the secret corresponding to the given address.
pub fn get_secret(address: Address) -> crate::Result<Secret> {
    let mut guard = Guard::default();

    guard.authenticate(&format!(
        "export private key for {address:#} [DANGEROUS!!!]"
    ))?;
    guard.get_secret_internal(address)
}

/// Opens an MacOS authentication prompt and on success, it uses the secret corresponding to the given address
/// to sign the given data and return the signature.
pub async fn sign_message_async(address: Address, data: Vec<u8>) -> crate::Result<Signature> {
    let mut guard = Guard::default();

    guard.authenticate(&format!(
        "use {address:#} to sign a message: {message}",
        message = truncate_with_count(&String::from_utf8_lossy(&data), 30)
    ))?;

    let signer = guard.get_secret_internal(address)?.into_alloy_signer()?;

    signer
        .sign_message(&data)
        .await
        .map_err(crate::Error::MessageSigningFailed)
}

pub async fn sign_tx_async(
    address: Address,
    mut tx: TxEip1559,
    meta: TransactionMeta,
) -> crate::Result<Signed<TxEip1559>> {
    let mut guard = Guard::default();

    let auth_msg = meta.get_display_message(&tx);
    guard.authenticate(&format!(
        "{auth_msg} from {address:#} on network {}",
        tx.chain_id
    ))?;

    let signer = guard.get_secret_internal(address)?.into_alloy_signer()?;

    let signature = signer
        .sign_transaction_sync(&mut tx)
        .map_err(crate::Error::TxSigningFailed)?;
    let tx_signed = SignableTransaction::into_signed(tx, signature);

    Ok(tx_signed)
}

/// Returns the default keychain.
fn keychain() -> SecKeychain {
    SecKeychain::default().expect("SecKeychain::default() - accessing default keychain failed")
}

/// A guard that ensures authentication is done before accessing secrets.
#[derive(Default)]
struct Guard {
    authenticated: bool,
}

impl Guard {
    /// Returns the secret corresponding to the given address.
    ///
    /// # Safety
    /// This method should only be called after authentication. It bypasses the authentication check amd is
    /// therefore restricted to internal use only in this module. These checks are added here to ensure that
    /// any future additions in this crate do not accidently call get_secret_inner without authenticating first.
    #[track_caller]
    fn get_secret_internal(&mut self, address: Address) -> crate::Result<Secret> {
        if !self.authenticated {
            panic!("get_secret_inner called without authentication");
        }

        // Reset authentication state after fetching the secret once.
        // User needs to authenticate again for subsequent operations.
        self.authenticated = false;

        let mnemonic_signer = || {
            let mnemonic_service = format!("gm:mnemonic:{address}");
            keychain()
                .find_generic_password(&mnemonic_service, &address.to_string())
                .map_err(|e| crate::Error::AccountNotFoundInKeychain(address, e))
                .and_then(|(pswd, _item)| {
                    String::from_utf8(pswd.to_vec())
                        .map_err(|e| {
                            crate::Error::ParsingStringFromKeychainSecretFailed(address, e)
                        })
                        .map(Secret::Mnemonic)
                })
        };
        let pk_signer = || {
            let pk_service = format!("gm:{address}");
            keychain()
                .find_generic_password(&pk_service, &address.to_string())
                .map_err(|e| crate::Error::AccountNotFoundInKeychain(address, e))
                .and_then(|(pswd, _item)| {
                    let raw_bytes = pswd.to_vec();
                    let hex_decoded = hex::decode(&raw_bytes);
                    let pk = hex_decoded.unwrap_or(raw_bytes);

                    SigningKey::from_slice(&pk)
                        .map_err(|e| crate::Error::PrivateKeyInvalid(address, e))
                        .map(|key| Secret::PrivateKey(key.to_bytes()))
                })
        };

        mnemonic_signer().or(pk_signer())
    }

    fn authenticate(&mut self, msg: &str) -> crate::Result<()> {
        auth::authenticate(msg)?;
        self.authenticated = true;
        Ok(())
    }
}

fn simplify_dict(dict: &CFDictionary) -> HashMap<String, String> {
    unsafe {
        let mut retmap = HashMap::new();
        let (keys, values) = dict.get_keys_and_values();
        for (k, v) in keys.iter().zip(values.iter()) {
            let keycfstr = CFString::wrap_under_get_rule((*k).cast());
            let val: String = match CFGetTypeID(*v) {
                cfstring if cfstring == CFString::type_id() => {
                    format!("{}", CFString::wrap_under_get_rule((*v).cast()))
                }
                cfdata if cfdata == CFData::type_id() => {
                    let buf = CFData::wrap_under_get_rule((*v).cast());
                    let mut vec = Vec::new();
                    vec.extend_from_slice(buf.bytes());
                    format!("{}", String::from_utf8_lossy(&vec))
                }
                cfdate if cfdate == CFDate::type_id() => format!(
                    "{}",
                    CFString::wrap_under_create_rule(CFCopyDescription(*v))
                ),
                _ => String::from("unknown"),
            };
            retmap.insert(format!("{keycfstr}"), val);
        }
        retmap
    }
}

#[cfg(test)]
mod test {

    #[test]
    #[ignore]
    fn see_all_accounts() {
        let list = super::get_account_list();

        println!("{list:#?}");
        panic!();
    }
}
