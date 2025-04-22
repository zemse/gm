use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use crate::error::Error;
use alloy::{
    hex,
    primitives::{address, Address, U256},
    signers::{
        k256::{ecdsa::SigningKey, FieldBytes},
        local::{MnemonicBuilder, PrivateKeySigner},
        utils::secret_key_to_address,
    },
};
use coins_bip39::{English, Mnemonic};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

pub trait AccountUtils {
    fn store_mnemonic_wallet(phrase: &str, address: Address) -> crate::Result<()>;

    fn store_private_key(private_key: &FieldBytes, address: Address) -> crate::Result<()>;

    fn get_account_list() -> Vec<Address>;

    fn get_secret(address: &Address) -> crate::Result<Secret>;
}

pub struct AccountManager;

impl AccountManager {
    pub fn create_mnemonic_wallet() -> crate::Result<Address> {
        let phrase = random_mnemonic()?;
        Self::import_mnemonic_wallet(&phrase)
    }

    pub fn import_mnemonic_wallet(phrase: &str) -> crate::Result<Address> {
        let address = get_address_from_mnemonic(phrase)?;
        Self::store_mnemonic_wallet(phrase, address)?;
        Ok(address)
    }

    pub fn import_private_key(private_key: &str) -> crate::Result<Address> {
        let private_key = hex::decode(private_key)?;
        let address = PrivateKeySigner::from_slice(&private_key)?.address();
        Self::store_private_key(FieldBytes::from_slice(private_key.as_slice()), address)?;
        Ok(address)
    }

    pub fn load_wallet(address: &Address) -> crate::Result<PrivateKeySigner> {
        match Self::get_secret(address)? {
            Secret::Mnemonic(phrase) => get_signer_from_mnemonic(&phrase),
            Secret::PrivateKey(private_key) => {
                Ok(PrivateKeySigner::from_slice(private_key.as_ref())?)
            }
        }
    }
}

impl AccountUtils for AccountManager {
    fn store_mnemonic_wallet(phrase: &str, address: Address) -> crate::Result<()> {
        #[cfg(target_os = "macos")]
        return macos::Macos::store_mnemonic_wallet(phrase, address);

        #[cfg(target_os = "linux")]
        return linux_insecure::LinuxInsecure::store_mnemonic_wallet(phrase, address);
    }

    fn store_private_key(private_key: &FieldBytes, address: Address) -> crate::Result<()> {
        #[cfg(target_os = "macos")]
        return macos::Macos::store_private_key(private_key, address);

        #[cfg(target_os = "linux")]
        return linux_insecure::LinuxInsecure::store_private_key(private_key, address);
    }

    fn get_account_list() -> Vec<Address> {
        #[cfg(target_os = "macos")]
        return macos::Macos::get_account_list();

        #[cfg(target_os = "linux")]
        return linux_insecure::LinuxInsecure::get_account_list();
    }

    fn get_secret(address: &Address) -> crate::Result<Secret> {
        #[cfg(target_os = "macos")]
        return macos::Macos::get_secret(address);

        #[cfg(target_os = "linux")]
        return linux_insecure::LinuxInsecure::get_secret(address);
    }
}

fn random_mnemonic() -> crate::Result<String> {
    let mnemonic = Mnemonic::<English>::new_with_count(&mut OsRng, 24)?;
    Ok(mnemonic.to_phrase())
}

fn get_signer_from_mnemonic(phrase: &str) -> crate::Result<PrivateKeySigner> {
    let signer = MnemonicBuilder::<English>::default()
        .phrase(phrase)
        .build()?;
    Ok(signer)
}

fn get_address_from_mnemonic(phrase: &str) -> crate::Result<Address> {
    let signer = get_signer_from_mnemonic(phrase)?;
    Ok(signer.address())
}

pub fn mine_wallet(
    mask_a: Address,
    mask_b: Address,
    max_dur: Option<Duration>,
) -> crate::Result<(Option<SigningKey>, usize, Duration)> {
    let address_one = address!("0xffffffffffffffffffffffffffffffffffffffff");
    let counter = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let result = Arc::new(Mutex::new(None));
    let start = Instant::now();

    rayon::scope(|s| {
        for _ in 0..rayon::current_num_threads() {
            let counter = Arc::clone(&counter);
            let stop = Arc::clone(&stop);
            let result = Arc::clone(&result);

            s.spawn(move |_| {
                // first private key is random
                let key = coins_bip32::prelude::SigningKey::random(&mut OsRng);
                let mut u = U256::from_be_slice(&key.to_bytes());

                while !stop.load(Ordering::Relaxed) {
                    if let Some(max_dur) = max_dur {
                        if Instant::now().duration_since(start) > max_dur {
                            break;
                        }
                    }

                    if let Ok(credential) =
                        SigningKey::from_bytes(FieldBytes::from_slice(&u.to_be_bytes_vec()))
                    {
                        let address = secret_key_to_address(&credential);
                        if address.bit_and(mask_a) == mask_a
                            && address.bit_xor(address_one).bit_and(mask_b) == mask_b
                        {
                            stop.store(true, Ordering::Relaxed);
                            let mut result = result.lock().unwrap();
                            *result = Some(credential);
                        };
                    } else {
                        // generate new random key
                    }
                    // change private key by one
                    u += U256::ONE;
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });

    let result = result.lock().unwrap().clone();
    let counter = counter.load(Ordering::Relaxed);
    Ok((result, counter, Instant::now().duration_since(start)))
}

#[derive(Clone, Debug)]
pub enum Secret {
    Mnemonic(String),
    PrivateKey(FieldBytes),
}

impl Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Secret::Mnemonic(mnemonic) => serializer.serialize_str(mnemonic),
            Secret::PrivateKey(private_key) => {
                let hex = hex::encode(private_key);
                serializer.serialize_str(&hex)
            }
        }
    }
}

impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() == 64 {
            let mut bytes = [0u8; 32];
            hex::decode_to_slice(&s, &mut bytes).map_err(serde::de::Error::custom)?;
            Ok(Secret::PrivateKey(*FieldBytes::from_slice(
                bytes.as_slice(),
            )))
        } else {
            Ok(Secret::Mnemonic(s))
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use core_foundation::{
        base::{CFCopyDescription, CFGetTypeID, TCFType},
        data::CFData,
        date::CFDate,
        dictionary::CFDictionary,
        string::CFString,
    };
    use security_framework::{
        item::{ItemClass, ItemSearchOptions, SearchResult},
        os::macos::keychain::SecKeychain,
    };
    use std::{collections::HashMap, str::FromStr};

    use super::*;

    fn keychain() -> SecKeychain {
        SecKeychain::default().expect("SecKeychain::default() - accessing default keychain failed")
    }

    pub struct Macos;

    impl AccountUtils for Macos {
        fn store_mnemonic_wallet(phrase: &str, address: Address) -> crate::Result<()> {
            let mnemonic_service = format!("gm:mnemonic:{address}");

            keychain()
                .add_generic_password(&mnemonic_service, &address.to_string(), phrase.as_bytes())
                .map_err(Error::AppleSecurityFrameworkError)?;

            Ok(())
        }

        fn store_private_key(private_key: &FieldBytes, address: Address) -> crate::Result<()> {
            let pk_service = format!("gm:{address}");

            keychain()
                .add_generic_password(
                    &pk_service,
                    &address.to_string(),
                    hex::encode(private_key).as_bytes(),
                )
                .map_err(Error::AppleSecurityFrameworkError)?;

            Ok(())
        }

        fn get_account_list() -> Vec<Address> {
            let mut search = ItemSearchOptions::default();
            search.class(ItemClass::generic_password());
            search.limit(100);
            search.load_attributes(true);

            let mut accounts = vec![];

            if let Ok(result) = search.search() {
                for item in result {
                    if let SearchResult::Dict(item) = item {
                        let item = simplify_dict(&item);
                        let service = item.get("svce");
                        if let Some(service) = service {
                            if service.starts_with("gm") {
                                let addr_str =
                                    item.get("acct").expect("must have an account address");
                                accounts.push(Address::from_str(addr_str).unwrap());
                            }
                        }
                    }
                }
            }

            accounts
        }

        fn get_secret(address: &Address) -> crate::Result<Secret> {
            let mnemonic_signer = || {
                let mnemonic_service = format!("gm:mnemonic:{address}");
                keychain()
                    .find_generic_password(&mnemonic_service, &address.to_string())
                    .map_err(crate::Error::from)
                    .and_then(|(pswd, _item)| {
                        String::from_utf8(pswd.to_vec())
                            .map_err(crate::Error::from)
                            .map(Secret::Mnemonic)
                    })
            };
            let pk_signer = || {
                let pk_service = format!("gm:{address}");
                keychain()
                    .find_generic_password(&pk_service, &address.to_string())
                    .map_err(crate::Error::from)
                    .and_then(|(pswd, _item)| {
                        let raw_bytes = pswd.to_vec();
                        let hex_decoded = hex::decode(&raw_bytes);
                        let pk = hex_decoded.unwrap_or(raw_bytes);

                        SigningKey::from_slice(&pk)
                            .map_err(crate::Error::from)
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
}

// #[cfg(target_os = "linux")]
mod linux_insecure {
    use crate::disk::{DiskInterface, FileFormat};

    use super::*;

    pub struct LinuxInsecure;

    impl AccountUtils for LinuxInsecure {
        fn store_mnemonic_wallet(phrase: &str, address: Address) -> crate::Result<()> {
            InsecurePrivateKeyStore::load().add(address, Secret::Mnemonic(phrase.to_string()));
            Ok(())
        }

        fn store_private_key(private_key: &FieldBytes, address: Address) -> crate::Result<()> {
            InsecurePrivateKeyStore::load().add(address, Secret::PrivateKey(*private_key));
            Ok(())
        }

        fn get_account_list() -> Vec<Address> {
            InsecurePrivateKeyStore::load().list()
        }

        fn get_secret(address: &Address) -> crate::Result<Secret> {
            InsecurePrivateKeyStore::load()
                .find_by_address(address)
                .ok_or(crate::Error::SecretNotFound(*address))
        }
    }

    // TODO remove this once we have implemented a secure store for linux
    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct InsecurePrivateKeyStore {
        pub keys: Vec<(Address, Secret)>,
    }

    impl DiskInterface for InsecurePrivateKeyStore {
        const FILE_NAME: &'static str = "insecure_private_key_store";
        const FORMAT: FileFormat = FileFormat::TOML;
    }

    impl InsecurePrivateKeyStore {
        pub fn add(&mut self, address: Address, key: Secret) {
            self.keys.push((address, key));
            self.save();
        }

        pub fn find_by_address(&self, address: &Address) -> Option<Secret> {
            self.keys.iter().find_map(|(stored_address, key)| {
                if stored_address == address {
                    Some(key.clone())
                } else {
                    None
                }
            })
        }

        pub fn list(self) -> Vec<Address> {
            self.keys.into_iter().map(|(address, _)| address).collect()
        }
    }
}
