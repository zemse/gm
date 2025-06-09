use std::{fmt, fmt::Debug, fs, path::PathBuf};

use alloy::primitives::Address;
use directories::BaseDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::Error;

pub enum FileFormat {
    TOML,
    YAML,
}

pub trait DiskInterface
where
    Self: Sized + Debug + Default + Serialize + DeserializeOwned,
{
    const FILE_NAME: &'static str;
    const FORMAT: FileFormat;

    /// Get the path to the file
    fn path() -> crate::Result<PathBuf> {
        let dirs =
            BaseDirs::new().ok_or(Error::InternalErrorStr("Failed to get base directories"))?;
        let path = dirs
            .home_dir()
            .join(".gm")
            .join(Self::FILE_NAME)
            .with_extension(match Self::FORMAT {
                FileFormat::TOML => "toml".to_string(),
                FileFormat::YAML => "yaml".to_string(),
            });
        Ok(path)
    }

    /// Load the content from the file
    fn load() -> crate::Result<Self> {
        let path = Self::path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            match Self::FORMAT {
                FileFormat::TOML => toml::from_str(&content).map_err(Error::from),
                FileFormat::YAML => serde_yaml::from_str(&content).map_err(Error::from),
            }
            .map_err(|err| {
                Error::DiskError(format!(
                    "Err({err:?}) while deserializing content at {path:?}"
                ))
            })
        } else {
            Ok(Self::default())
        }
    }

    /// Save settings to a file
    fn save(&self) -> crate::Result<()> {
        let path = Self::path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?; // Ensure config directory exists
        }

        let content = match Self::FORMAT {
            FileFormat::TOML => toml::to_string_pretty(self).map_err(Error::from),
            FileFormat::YAML => serde_yaml::to_string(self).map_err(Error::from),
        }
        .map_err(|err| {
            Error::DiskError(format!("Err({err:?}) while serializing {path:?}: {self:?}"))
        })?;

        fs::write(path, content)?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AddressBook {
    entries: Vec<AddressBookEntry>,
}

impl DiskInterface for AddressBook {
    const FILE_NAME: &'static str = "address_book";
    const FORMAT: FileFormat = FileFormat::YAML;
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AddressBookEntry {
    pub name: String,
    pub address: Address,
    // TODO we can add more fields here like last interacted time
}

impl AddressBook {
    pub fn add(&mut self, entry: AddressBookEntry) -> Result<(), Error> {
        if self.find_by_name(&entry.name).is_some() {
            return Err(Error::AddressBook("Name already exists in the addressbook"));
        }

        if self.find_by_address(&entry.address).is_some() {
            return Err(Error::AddressBook(
                "Address already exists in the addressbook",
            ));
        }

        self.entries.push(entry);
        self.save()?;

        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> crate::Result<()> {
        self.entries.remove(index);
        self.save()
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
    ) -> crate::Result<Option<(usize, AddressBookEntry)>> {
        if let Some(address) = address {
            Ok(self.find_by_address(address))
        } else if let Some(name) = name {
            Ok(self.find_by_name(name))
        } else if let Some(id) = id {
            let index = *id - 1;
            let entry = AddressBook::load()?.list()[index].clone();
            Ok(Some((*id, entry)))
        } else {
            Ok(None)
        }
    }

    pub fn update(&mut self, id: usize, new_entry: AddressBookEntry) -> crate::Result<()> {
        self.entries[id - 1] = new_entry;
        self.save()
    }

    pub fn list(&self) -> &Vec<AddressBookEntry> {
        &self.entries
    }

    pub fn list_owned(self) -> Vec<AddressBookEntry> {
        self.entries
    }

    pub fn load_list() -> crate::Result<Vec<AddressBookEntry>> {
        Ok(AddressBook::load()?.list_owned())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub current_account: Option<Address>,
    pub testnet_mode: bool,
    #[serde(default)]
    pub developer_mode: bool,
    pub alchemy_api_key: Option<String>,
}

impl DiskInterface for Config {
    const FILE_NAME: &'static str = "config";
    const FORMAT: FileFormat = FileFormat::TOML;
}

impl Config {
    pub fn current_account() -> crate::Result<Option<Address>> {
        Ok(Config::load()?.current_account)
    }

    pub fn set_current_account(address: Address) -> crate::Result<()> {
        let mut config = Config::load()?;
        config.current_account = Some(address);
        config.save()?;
        Ok(())
    }

    pub fn alchemy_api_key() -> crate::Result<String> {
        Config::load()?
            .alchemy_api_key
            .ok_or(crate::Error::AlchemyApiKeyNotSet)
    }

    pub fn set_alchemy_api_key(alchemy_api_key: String) -> crate::Result<()> {
        let mut config = Config::load()?;
        config.alchemy_api_key = Some(alchemy_api_key);
        config.save()?;
        Ok(())
    }
}

// Implementing Display for AddressBookEntry to format how entries appear in selections.
impl fmt::Display for AddressBookEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.address)
    }
}
