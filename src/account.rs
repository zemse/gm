use std::{collections::HashMap, str::FromStr};

use alloy::{
    primitives::Address,
    signers::{k256::ecdsa::SigningKey, local::PrivateKeySigner},
};
use inquire::Select;
use rand::rngs::OsRng;
use security_framework::{
    item::{ItemClass, ItemSearchOptions, SearchResult},
    os::macos::keychain::SecKeychain,
};

fn keychain() -> SecKeychain {
    SecKeychain::default().expect("SecKeychain::default() - accessing default keychain failed")
}

fn address_to_service(address: &Address) -> String {
    format!("gm:{address}")
}

/// Create a new private key wallet and store it in the keychain
pub fn create_privatekey_wallet() -> Address {
    let key = SigningKey::random(&mut OsRng);
    let private_key = key.to_bytes();
    let signer = PrivateKeySigner::from(key);
    let address = signer.address();

    keychain()
        .add_generic_password(
            &address_to_service(&address),
            &address.to_string(),
            private_key.as_slice(),
        )
        .unwrap();

    println!("Wallet created with address: {}", address);

    address
}

pub fn list_of_wallets() {
    let mut search = ItemSearchOptions::default();
    search.class(ItemClass::generic_password());
    search.limit(100);
    search.load_attributes(true);
    // search.load_data(true);

    let mut accounts = vec![];

    if let Ok(result) = search.search() {
        for item in result {
            if let SearchResult::Dict(item) = item {
                let item = simplify_dict(&item);
                let service = item.get("svce");
                if let Some(service) = service {
                    if service.starts_with("gm") {
                        let addr_str = item.get("acct").expect("must have an account address");
                        accounts.push(Address::from_str(addr_str).unwrap());
                    }
                }
            }
        }
    }

    let address = Select::new("Choose account to load:", accounts)
        .with_formatter(&|a| format!("{a}"))
        .prompt()
        .ok();

    if let Some(address) = address {
        let mut config = Config::load();
        config.current_account = address;
        config.save();

        // not necessary to do it right now, we can store this address in the state and later load it when we want to make a tranasaction
        // let result =
        //     keychain().find_generic_password(&address_to_service(&address), &address.to_string());
        // if let Ok((pswd, _item)) = result {
        //     let key = SigningKey::from_slice(pswd.as_ref())
        //         .expect("must create a valid signing key from keychain password");
        //     let signer = PrivateKeySigner::from(key);
        //     println!("Address: {:?}", signer.address());
        // }
    }
}

pub fn load_wallet() {}

// TODO move this to upstream
use core_foundation::{
    base::{CFCopyDescription, CFGetTypeID, TCFType},
    data::CFData,
    date::CFDate,
    dictionary::CFDictionary,
    string::CFString,
};

use crate::config::Config;
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
