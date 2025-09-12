use alloy::{
    hex,
    primitives::Address,
    signers::k256::{ecdsa::SigningKey, FieldBytes},
};
use core_foundation::{
    base::{CFCopyDescription, CFGetTypeID, TCFType},
    data::CFData,
    date::CFDate,
    dictionary::CFDictionary,
    string::CFString,
};
use gm_common::secret::Secret;
use security_framework::{
    item::{ItemClass, ItemSearchOptions, SearchResult},
    os::macos::keychain::SecKeychain,
};
use std::collections::HashMap;

fn keychain() -> SecKeychain {
    SecKeychain::default().expect("SecKeychain::default() - accessing default keychain failed")
}

pub struct Macos;

impl Macos {
    pub fn store_mnemonic_wallet(phrase: &str, address: Address) -> crate::Result<()> {
        let mnemonic_service = format!("gm:mnemonic:{address}");

        keychain()
            .add_generic_password(&mnemonic_service, &address.to_string(), phrase.as_bytes())
            .map_err(|e| crate::Error::StoringAccountInKeychainFailed(address, e))?;

        Ok(())
    }

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

    pub fn get_secret(address: &Address) -> crate::Result<Secret> {
        let mnemonic_signer = || {
            let mnemonic_service = format!("gm:mnemonic:{address}");
            keychain()
                .find_generic_password(&mnemonic_service, &address.to_string())
                .map_err(|e| crate::Error::AccountNotFoundInKeychain(*address, e))
                .and_then(|(pswd, _item)| {
                    String::from_utf8(pswd.to_vec())
                        .map_err(|e| {
                            crate::Error::ParsingStringFromKeychainSecretFailed(*address, e)
                        })
                        .map(Secret::Mnemonic)
                })
        };
        let pk_signer = || {
            let pk_service = format!("gm:{address}");
            keychain()
                .find_generic_password(&pk_service, &address.to_string())
                .map_err(|e| crate::Error::AccountNotFoundInKeychain(*address, e))
                .and_then(|(pswd, _item)| {
                    let raw_bytes = pswd.to_vec();
                    let hex_decoded = hex::decode(&raw_bytes);
                    let pk = hex_decoded.unwrap_or(raw_bytes);

                    SigningKey::from_slice(&pk)
                        .map_err(|e| crate::Error::PrivateKeyInvalid(*address, e))
                        .map(|key| Secret::PrivateKey(key.to_bytes()))
                })
        };

        mnemonic_signer().or(pk_signer())
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
        let list = super::Macos::get_account_list();

        println!("{list:#?}");
        panic!();
    }
}
