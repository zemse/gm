use std::{fmt::Debug, fs, path::PathBuf};

use alloy::{hex, primitives::Address, signers::k256::FieldBytes};
use directories::BaseDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait DiskInterface
where
    Self: Sized + Debug + Default + Serialize + DeserializeOwned,
{
    const FILE_NAME: &'static str;

    /// Get the path to the file
    fn path() -> PathBuf {
        let dirs = BaseDirs::new().expect("Failed to get base directories");
        dirs.home_dir().join(".gm").join(Self::FILE_NAME)
    }

    /// Load settings from a file
    fn load() -> Self {
        let path = Self::path();

        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
            toml::from_str(&content).unwrap_or_else(|_| Self::default())
        } else {
            Self::default()
        }
    }

    /// Save settings to a file
    fn save(&self) {
        let path = Self::path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok(); // Ensure config directory exists
        }

        let content = toml::to_string_pretty(self).expect("Failed to serialize");
        fs::write(path, content).expect("Failed to write file");
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AddressBook {
    entries: Vec<AddressBookEntry>,
}

impl DiskInterface for AddressBook {
    const FILE_NAME: &'static str = "address_book.toml";
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AddressBookEntry {
    pub name: String,
    pub address: Address,
    // TODO we can add more fields here like last interacted time
}

impl AddressBook {
    pub fn add(&mut self, entry: AddressBookEntry) {
        self.entries.push(entry);
        self.save();
    }

    pub fn remove(&mut self, index: usize) {
        self.entries.remove(index);
        self.save();
    }

    pub fn find_by_address(&self, address: &Address) -> Option<(usize, AddressBookEntry)> {
        self.entries.iter().enumerate().find_map(|(index, entry)| {
            if &entry.address == address {
                Some((index, entry.clone()))
            } else {
                None
            }
        })
    }

    pub fn find_by_name(&self, name: &str) -> Option<(usize, AddressBookEntry)> {
        self.entries.iter().enumerate().find_map(|(index, entry)| {
            if entry.name == name {
                Some((index, entry.clone()))
            } else {
                None
            }
        })
    }

    pub fn find(
        &self,
        id: &Option<usize>,
        address: &Option<Address>,
        name: &Option<&String>,
    ) -> Option<(usize, AddressBookEntry)> {
        if let Some(address) = address {
            self.find_by_address(address)
        } else if let Some(name) = name {
            self.find_by_name(name)
        } else if let Some(id) = id {
            let index = *id - 1;
            let entry = AddressBook::load().list()[index].clone();
            Some((*id, entry))
        } else {
            None
        }
    }

    pub fn update(&mut self, id: usize, new_entry: AddressBookEntry) {
        self.entries[id - 1] = new_entry;
        self.save();
    }

    pub fn list(&self) -> &Vec<AddressBookEntry> {
        &self.entries
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub current_account: Address, // TODO change to Option<Address>
    pub debug_mode: bool,
    pub alchemy_api_key: Option<String>,
}

impl DiskInterface for Config {
    const FILE_NAME: &'static str = "config.toml";
}

// TODO remove this once we have implemented a secure store for linux
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct InsecurePrivateKeyStore {
    pub keys: Vec<(Address, String)>,
}

impl DiskInterface for InsecurePrivateKeyStore {
    const FILE_NAME: &'static str = "insecure_private_key_store.toml";
}

impl InsecurePrivateKeyStore {
    pub fn add(&mut self, address: Address, key: FieldBytes) {
        self.keys.push((address, hex::encode_prefixed(key)));
        self.save();
    }

    pub fn find_by_address(&self, address: &Address) -> Option<FieldBytes> {
        self.keys.iter().find_map(|(stored_address, key)| {
            if stored_address == address {
                hex::decode(key)
                    .ok()
                    .map(|d| *FieldBytes::from_slice(d.as_slice()))
            } else {
                None
            }
        })
    }

    pub fn list(&self) -> Vec<&Address> {
        self.keys.iter().map(|(address, _)| address).collect()
    }
}
