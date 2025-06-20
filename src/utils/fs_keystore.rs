use crate::utils::account::Secret;
use crate::{Error, Result};
use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use rand::rngs::OsRng;
use std::{fs, path::PathBuf};
//use alloy::signers::keystore::KeystoreSigner;
use alloy::signers::local::LocalSigner;




pub struct FsKeystore;

impl FsKeystore {
    fn keystore_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| Error::InternalErrorStr("HOME environment variable not set"))?;
        Ok(PathBuf::from(home).join(".gm").join("keystores"))
    }

    pub fn init_dir() -> Result<PathBuf> {
        let dir = Self::keystore_dir()?;
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
                if let Ok(data) = fs::read_to_string(&p) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                        if let Some(addr_str) = json.get("address").and_then(|v| v.as_str()) {
                            // Ethereum V3 keystores Standard
                            let addr = format!("0x{}", addr_str);
                            if let Ok(addr) = addr.parse::<Address>() {
                                out.push(addr);
                            }
                        }
                    }
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
        let dir = Self::keystore_dir()?;
        fs::create_dir_all(&dir)?;
        let file = dir.join(format!("{}_mnemonic.txt", address));
        fs::write(file, phrase)?;
        Ok(())
    }
}
