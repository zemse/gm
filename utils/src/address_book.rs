use std::fmt;

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::disk_storage::{DiskStorageInterface, FileFormat};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AddressBookStore {
    entries: Vec<AddressBookEntry>,
}

impl DiskStorageInterface for AddressBookStore {
    const FILE_NAME: &'static str = "address_book";
    const FORMAT: FileFormat = FileFormat::YAML;
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AddressBookEntry {
    pub name: String,
    pub address: Address,
    // TODO we can add more fields here like last interacted time
}

// Implementing Display for AddressBookEntry to format how entries appear in selections.
impl fmt::Display for AddressBookEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.address)
    }
}

impl AddressBookStore {
    pub fn add(&mut self, entry: AddressBookEntry) -> crate::Result<()> {
        if self.find_by_name(&entry.name).is_some() {
            return Err(crate::Error::AddressBookNameExists(entry.name.clone()));
        }

        if self.find_by_address(&entry.address).is_some() {
            return Err(crate::Error::AddressBookAddressExists(entry.address));
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
            let entry = AddressBookStore::load()?.list()[index].clone();
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
        Ok(AddressBookStore::load()?.list_owned())
    }
}
