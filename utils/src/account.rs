use alloy::{
    hex,
    primitives::{keccak256, Address},
    signers::{
        k256::{
            ecdsa::SigningKey, elliptic_curve::sec1::ToEncodedPoint, AffinePoint, FieldBytes,
            ProjectivePoint, Scalar,
        },
        local::{MnemonicBuilder, PrivateKeySigner},
    },
};
use coins_bip39::{English, Mnemonic};
use gm_common::secret::Secret;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

pub trait AccountUtils {
    fn store_mnemonic_wallet(phrase: &str, address: Address) -> crate::Result<()>;

    fn store_private_key(private_key: &FieldBytes, address: Address) -> crate::Result<()>;

    fn get_account_list() -> crate::Result<Vec<Address>>;

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
        return Ok(gm_macos::Macos::store_mnemonic_wallet(phrase, address)?);

        #[cfg(target_os = "linux")]
        return linux_insecure::LinuxInsecure::store_mnemonic_wallet(phrase, address);
    }

    fn store_private_key(private_key: &FieldBytes, address: Address) -> crate::Result<()> {
        #[cfg(target_os = "macos")]
        return Ok(gm_macos::Macos::store_private_key(private_key, address)?);

        #[cfg(target_os = "linux")]
        return linux_insecure::LinuxInsecure::store_private_key(private_key, address);
    }

    fn get_account_list() -> crate::Result<Vec<Address>> {
        #[cfg(target_os = "macos")]
        return Ok(gm_macos::Macos::get_account_list()?);

        #[cfg(target_os = "linux")]
        return linux_insecure::LinuxInsecure::get_account_list();
    }

    fn get_secret(address: &Address) -> crate::Result<Secret> {
        #[cfg(target_os = "macos")]
        return Ok(gm_macos::Macos::get_secret(address)?);

        #[cfg(target_os = "linux")]
        return linux_insecure::LinuxInsecure::get_secret(address);
    }
}

fn random_mnemonic() -> crate::Result<String> {
    let mnemonic = Mnemonic::<English>::new_with_count(&mut OsRng, 24)
        .map_err(crate::Error::MnemonicGenerationFailed)?;
    Ok(mnemonic.to_phrase())
}

fn get_signer_from_mnemonic(phrase: &str) -> crate::Result<PrivateKeySigner> {
    let signer = MnemonicBuilder::<English>::default()
        .phrase(phrase)
        .build()
        .map_err(crate::Error::MnemonicSignerFailed)?;
    Ok(signer)
}

fn get_address_from_mnemonic(phrase: &str) -> crate::Result<Address> {
    let signer = get_signer_from_mnemonic(phrase)?;
    Ok(signer.address())
}

/// Mines a wallet whose address matches the given masks.
/// - `mask_a` specifies bits that must be 1 in the 20 byte address.
/// - `mask_b` specifies bits that must be 0 in the 20 byte address.
/// - `max_dur` specifies the maximum duration to mine for. If None, mines indefinitely.
/// - `shutdown_signal` can be used to stop mining from main thread.
pub fn mine_wallet(
    mask_a: Address,
    mask_b: Address,
    max_dur: Option<Duration>,
    exit_signal: Arc<AtomicBool>,
) -> crate::Result<(Option<SigningKey>, u64, Duration)> {
    let counter = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let result = Arc::new(Mutex::new(None));
    let start = Instant::now();

    rayon::scope(|s| {
        for _ in 0..rayon::current_num_threads() {
            let counter = Arc::clone(&counter);
            let stop = Arc::clone(&stop);
            let result = Arc::clone(&result);
            let exit_signal = exit_signal.clone();
            s.spawn(move |_| {
                // Start with a random private key
                let initial_random_key = **SigningKey::random(&mut OsRng).as_nonzero_scalar();

                let mut k = initial_random_key;
                let mut curve_point = ProjectivePoint::GENERATOR * k;

                while !stop.load(Ordering::Relaxed) && !exit_signal.load(Ordering::Relaxed) {
                    if let Some(max_dur) = max_dur {
                        if Instant::now().duration_since(start) > max_dur {
                            break;
                        }
                    }

                    let address = eth_addr_from_affine(&curve_point.to_affine());
                    if addr_match(&address, &mask_a, &mask_b) {
                        stop.store(true, Ordering::Relaxed);
                        let mut result = result.lock().unwrap();
                        *result = Some(SigningKey::from_bytes(&k.to_bytes()).unwrap());
                        break;
                    };

                    // Change private key by one
                    k += Scalar::ONE;
                    curve_point += ProjectivePoint::GENERATOR;
                }

                let count_bytes: [u8; 32] = (k - initial_random_key).to_bytes().into();
                let count = u64::from_be_bytes(count_bytes[24..32].try_into().unwrap());
                counter.fetch_add(count, Ordering::Relaxed);
            });
        }
    });

    let result = result.lock().unwrap().clone();
    let counter = counter.load(Ordering::Relaxed);
    Ok((result, counter, Instant::now().duration_since(start)))
}

#[inline(always)]
fn addr_match(addr: &[u8; 20], mask_a: &[u8; 20], mask_b: &[u8; 20]) -> bool {
    for i in 0..20 {
        if (addr[i] & mask_a[i]) != mask_a[i] {
            return false;
        }
        if ((!addr[i]) & mask_b[i]) != mask_b[i] {
            return false;
        }
    }
    true
}

#[inline(always)]
fn eth_addr_from_affine(aff: &AffinePoint) -> [u8; 20] {
    let ep = aff.to_encoded_point(false);
    let bytes = ep.as_bytes(); // 65
    let out = keccak256(&bytes[1..]);
    out[12..32].try_into().unwrap()
}

pub mod linux_insecure {
    use crate::disk_storage::{DiskStorageInterface, FileFormat};

    use super::*;

    pub struct LinuxInsecure;

    impl AccountUtils for LinuxInsecure {
        fn store_mnemonic_wallet(phrase: &str, address: Address) -> crate::Result<()> {
            InsecurePrivateKeyStore::load()?.add(address, Secret::Mnemonic(phrase.to_string()))
        }

        fn store_private_key(private_key: &FieldBytes, address: Address) -> crate::Result<()> {
            InsecurePrivateKeyStore::load()?.add(address, Secret::PrivateKey(*private_key))
        }

        fn get_account_list() -> crate::Result<Vec<Address>> {
            Ok(InsecurePrivateKeyStore::load()?.list())
        }

        fn get_secret(address: &Address) -> crate::Result<Secret> {
            InsecurePrivateKeyStore::load()?
                .find_by_address(address)
                .ok_or(crate::Error::SecretNotFound(*address))
        }
    }

    // TODO remove this once we have implemented a secure store for linux
    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct InsecurePrivateKeyStore {
        pub keys: Vec<(Address, Secret)>,
    }

    impl DiskStorageInterface for InsecurePrivateKeyStore {
        const FILE_NAME: &'static str = "insecure_private_key_store";
        const FORMAT: FileFormat = FileFormat::TOML;
    }

    impl InsecurePrivateKeyStore {
        pub fn add(&mut self, address: Address, key: Secret) -> crate::Result<()> {
            self.keys.push((address, key));
            self.save()
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
