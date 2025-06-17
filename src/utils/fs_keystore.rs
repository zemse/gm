use rand::rngs::OsRng;
use std::{fs, path::PathBuf};
use alloy::primitives::Address;
use crate::{Result};
use crate::utils::account::{Secret};
use alloy_signer_local::PrivateKeySigner; 


pub struct FsKeystore;

impl FsKeystore {
    fn keystore_dir() -> PathBuf {
        dirs::home_dir().expect("home!") .join(".gm").join("keystores")
    }

    pub fn init_dir() -> Result<PathBuf> {
        let dir = Self::keystore_dir();
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn store_private_key(raw_pk: &[u8], password: &str, name: Option<&str>) -> Result<Address> {
        let dir = Self::init_dir()?;
        let (signer, _file) = PrivateKeySigner::new_keystore(dir, &mut OsRng, password, name)?;
        Ok(signer.address())
    }

    pub fn list_addresses() -> Result<Vec<Address>> {
        let dir = Self::init_dir()?;
        let mut out = Vec::new();
        for entry in fs::read_dir(dir)? {
            let p = entry?.path();
            if p.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(signer) = PrivateKeySigner::decrypt_keystore(&p, "") {
                    out.push(signer.address());
                }
            }
        }
        Ok(out)
    }

    pub fn load_secret(addr: &Address, password: &str) -> Result<Secret> {
        for entry in fs::read_dir(Self::init_dir()?)? {
            let p = entry?.path();
            if let Ok(signer) = PrivateKeySigner::decrypt_keystore(&p, password) {
                if signer.address() == *addr {
                    return Ok(Secret::PrivateKey(signer.to_bytes().into()));
                }
            }
        }
        Err(crate::Error::SecretNotFound(*addr))
    }
    pub fn store_mnemonic_wallet(phrase: &str, address: Address) -> Result<()> {
        // For now, just store as a plaintext file named "{address}_mnemonic.txt"
        // In production, you should encrypt this!
        let dir = dirs::home_dir().unwrap().join(".gm").join("keystores");
        std::fs::create_dir_all(&dir)?;
        let file = dir.join(format!("{}_mnemonic.txt", address));
        std::fs::write(file, phrase)?;
        Ok(())
    }
}
