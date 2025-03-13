use std::{collections::HashMap, str::FromStr};

use crate::{
    disk::{Config, DiskInterface, InsecurePrivateKeyStore},
    error::Error,
    impl_inquire_selection,
    traits::Handle,
};

use alloy::{
    primitives::{bytes::BytesMut, keccak256, Address, Bytes, U256},
    signers::{
        k256::{ecdsa::SigningKey, FieldBytes},
        local::PrivateKeySigner,
    },
};
use clap::{command, Subcommand};
use core_foundation::{
    base::{CFCopyDescription, CFGetTypeID, TCFType},
    data::CFData,
    date::CFDate,
    dictionary::CFDictionary,
    string::CFString,
};
use inquire::{Password, Select};
use rand::{rngs::OsRng, RngCore};
use security_framework::{
    item::{ItemClass, ItemSearchOptions, SearchResult},
    os::macos::keychain::SecKeychain,
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Subcommand, Display, EnumIter)]
pub enum AccountActions {
    #[command(alias = "new")]
    Create,

    #[command(alias = "ls")]
    List,
}

impl_inquire_selection!(AccountActions, ());

impl Handle for AccountActions {
    fn handle(&self, _carry_on: ()) {
        match self {
            AccountActions::List => {
                println!("Listing all accounts...");
                list_of_wallets();
            }
            AccountActions::Create => {
                println!("Creating a new account...");
                create_privatekey_wallet();
            }
        }
    }
}

fn keychain() -> SecKeychain {
    SecKeychain::default().expect("SecKeychain::default() - accessing default keychain failed")
}

fn address_to_service(address: &Address) -> String {
    format!("gm:{address}")
}

fn gen_wallet() -> (FieldBytes, PrivateKeySigner, Address) {
    // TODO include random string to strengthen the key and then hash it all
    // TODO store mnemonic phrase
    // TODO this user input, if we can show some kind of progress bar with security, it can incentivise
    // people to enter more words in this.

    // Take some user input to improve the private key
    let user_input = Password::new(
        "Enter things you see around you or type any random text and then press enter:",
    )
    .without_confirmation()
    .prompt();

    // generate the private key
    let private_key = if let Ok(mut user_input) = user_input {
        if user_input.len() % 2 != 0 {
            user_input.push('t');
        }
        let user_input = Bytes::copy_from_slice(user_input.as_bytes());

        loop {
            let mut random_value = [0u8; 32];
            OsRng.fill_bytes(&mut random_value);
            let random_value = U256::from_be_bytes(random_value);

            let mut concat = BytesMut::with_capacity(32 + user_input.len());
            concat.extend_from_slice(&random_value.to_be_bytes::<32>());
            concat.extend_from_slice(&user_input);
            let result = keccak256(concat);

            let result = SigningKey::from_slice(result.as_slice());
            if let Ok(key) = result {
                break key;
            }
        }
    } else {
        SigningKey::random(&mut OsRng)
    };

    let private_key_bytes = private_key.to_bytes();
    let signer = PrivateKeySigner::from(private_key);
    let address = signer.address();

    (private_key_bytes, signer, address)
}

/// Create a new private key wallet and store it in the keychain
pub fn create_privatekey_wallet_macos() -> Address {
    let (private_key_bytes, _signer, address) = gen_wallet();

    keychain()
        .add_generic_password(
            &address_to_service(&address),
            &address.to_string(),
            private_key_bytes.as_slice(),
        )
        .unwrap();

    println!("Wallet created with address: {}", address);
    println!("The private key is stored securely in your keychain.");

    address
}

pub fn list_of_wallets_macos() {
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
    }
}

pub fn load_wallet_macos(address: Address) -> Result<PrivateKeySigner, Error> {
    println!("Unlocking wallet {:?}", address);
    Ok(keychain()
        .find_generic_password(&address_to_service(&address), &address.to_string())
        .map(|(pswd, _item)| {
            let key = SigningKey::from_slice(pswd.as_ref())
                .expect("must create a valid signing key from keychain password");
            PrivateKeySigner::from(key)
        })?)
}

pub fn create_privatekey_wallet_linux_insecure() -> Address {
    let (private_key_bytes, _signer, address) = gen_wallet();

    let mut store = InsecurePrivateKeyStore::load();
    store.add(address, private_key_bytes);
    store.save();

    println!("Wallet created with address: {}", address);
    println!("The private key is stored insecurely in your filesystem.");

    address
}

pub fn list_of_wallets_linux_insecure() {
    let store = InsecurePrivateKeyStore::load();
    let accounts = store.list();

    let address = Select::new("Choose account to load:", accounts)
        .with_formatter(&|a| format!("{a}"))
        .prompt()
        .ok();

    if let Some(address) = address {
        let mut config = Config::load();
        config.current_account = *address;
        config.save();
    }
}

pub fn load_wallet_linux_insecure(address: Address) -> Result<PrivateKeySigner, Error> {
    println!("Unlocking wallet {:?}", address);
    let store = InsecurePrivateKeyStore::load();
    let key = store
        .find_by_address(&address)
        .expect("must find key in store");
    Ok(SigningKey::from_slice(key.as_slice()).map(PrivateKeySigner::from_signing_key)?)
}

pub fn create_privatekey_wallet() -> Address {
    #[cfg(target_os = "macos")]
    let address = create_privatekey_wallet_macos();
    #[cfg(target_os = "linux")]
    let address = create_privatekey_wallet_linux_insecure();
    address
}

pub fn list_of_wallets() {
    #[cfg(target_os = "macos")]
    list_of_wallets_macos();
    #[cfg(target_os = "linux")]
    list_of_wallets_linux_insecure();
}

pub fn load_wallet(address: Address) -> Result<PrivateKeySigner, Error> {
    #[cfg(target_os = "macos")]
    let key = load_wallet_macos(address);
    #[cfg(target_os = "linux")]
    let key = load_wallet_linux_insecure(address);
    key
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
