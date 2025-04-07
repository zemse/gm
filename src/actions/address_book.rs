use std::fmt::Display;
use qrcode::QrCode;
use qrcode::render::unicode;

use crate::{
    disk::{AddressBook, AddressBookEntry, DiskInterface},
    impl_inquire_selection,
    utils::{Handle, Inquire},
};

use alloy::{hex::FromHex, primitives::Address};
use clap::Subcommand;
use inquire::{Text, Select};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Subcommand)]
pub enum AddressBookActions {
    #[command(alias = "new")]
    Create {
        address: Option<Address>,
        name: Option<String>,
    },

    #[command(alias = "v")]
    View {
        id: Option<usize>,
        address: Option<Address>,
        name: Option<String>,
    },
}

impl Display for AddressBookActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressBookActions::Create { .. } => write!(f, "Create new address book entry"),
            AddressBookActions::View { id, address, name } => {
                let (id, entry) = AddressBook::load()
                    .find(id, address, &name.as_ref())
                    .expect("entry not found");

                write!(f, "{}) {} - {}", id, entry.name, entry.address)
            }
        }
    }
}


impl Inquire for AddressBookActions {
    fn inquire(_: &()) -> Option<AddressBookActions> {
        let options = vec![AddressBookActions::Create {
            address: None,
            name: None,
        }]
        .into_iter()
        .chain(
            (0..AddressBook::load().list().len()).map(|index| AddressBookActions::View {
                address: None,
                name: None,
                id: Some(index + 1),
            }),
        )
        .collect::<Vec<AddressBookActions>>();

        inquire::Select::new("Select entry:", options).prompt().ok()
    }
}

impl Handle for AddressBookActions {
    fn handle(&self, _carry_on: ()) {
        match self {
            AddressBookActions::Create { address, name } => {
                let address = address
                    .or_else(|| {
                        Some(
                            Text::new("Enter address")
                                .prompt()
                                .expect("must input address")
                                .parse()
                                .expect("failed to parse address"),
                        )
                    })
                    .expect("must have an address");

                let name = name
                    .clone()
                    .or_else(|| Some(Text::new("Enter name").prompt().expect("must input name")))
                    .expect("must have a name");

                if name.is_empty() {
                    panic!("Name must be at least 1 characters long");
                }

                AddressBook::load().add(AddressBookEntry { name, address });
                println!("Entry added to address book");
            }
            AddressBookActions::View { id, address, name } => {
                AddressBookViewActions::handle_optn_inquire(
                    &None,
                    AddressBookViewCarryOn {
                        id: *id,
                        address: *address,
                        name: name.clone(),
                    },
                );
            }
        }
    }
}

#[derive(Subcommand, Display, EnumIter)]
pub enum AddressBookViewActions {
    #[command(alias = "cn")]
    #[strum(serialize = "Change Name")]
    ChangeName {
        id: Option<usize>,
        address: Option<Address>,
        name: Option<String>,
        new_name: Option<String>,
    },

    #[command(alias = "ca")]
    #[strum(serialize = "Change Address")]
    ChangeAddress {
        id: Option<usize>,
        address: Option<Address>,
        name: Option<String>,
        new_address: Option<Address>,
    },

    #[command(alias = "d")]
    Delete {
        id: Option<usize>,
        address: Option<Address>,
        name: Option<String>,
    },
    #[command(alias = "qr")]
    ShowQRCode { 
        id: Option<usize>,
        address: Option<Address>,
        name: Option<String>,
     },
}

pub struct AddressBookViewCarryOn {
    id: Option<usize>,
    address: Option<Address>,
    name: Option<String>,
}

impl_inquire_selection!(AddressBookViewActions, AddressBookViewCarryOn);

impl Handle<AddressBookViewCarryOn> for AddressBookViewActions {
    fn handle(&self, carry_on: AddressBookViewCarryOn) {
        match self {
            AddressBookViewActions::ChangeName {
                id,
                address,
                name,
                new_name,
            } => {
                let (id, entry) = AddressBook::load()
                    .find(
                        &id.or(carry_on.id),
                        &address.or(carry_on.address),
                        &name.as_ref().or(carry_on.name.as_ref()),
                    )
                    .expect("entry not found");

                let new_name = new_name
                    .clone()
                    .or_else(|| {
                        Some(
                            Text::new("Enter new name")
                                .with_initial_value(&entry.name)
                                .prompt()
                                .expect("must input new name"),
                        )
                    })
                    .expect("must have a new name");

                if new_name.is_empty() {
                    panic!("Name must be at least 1 characters long");
                }

                AddressBook::load().update(
                    id,
                    AddressBookEntry {
                        name: new_name,
                        address: entry.address,
                    },
                );

                println!("Entry updated in address book");
            }
            AddressBookViewActions::ChangeAddress {
                id,
                address,
                name,
                new_address,
            } => {
                let (id, entry) = AddressBook::load()
                    .find(
                        &id.or(carry_on.id),
                        &address.or(carry_on.address),
                        &name.as_ref().or(carry_on.name.as_ref()),
                    )
                    .expect("entry not found");

                let new_address = new_address
                    .or_else(|| {
                        Some(
                            Text::new("Enter new address")
                                .with_initial_value(&entry.address.to_string())
                                .prompt()
                                .expect("must input new address")
                                .parse()
                                .expect("failed to parse address"),
                        )
                    })
                    .expect("must have a new address");

                let new_address = Address::from_hex(new_address).expect("error parsing hex string");

                AddressBook::load().update(
                    id,
                    AddressBookEntry {
                        name: entry.name,
                        address: new_address,
                    },
                );

                println!("Entry updated in address book");
            }
            AddressBookViewActions::Delete { id, address, name } => {
                let (id, _) = AddressBook::load()
                    .find(
                        &id.or(carry_on.id),
                        &address.or(carry_on.address),
                        &name.as_ref().or(carry_on.name.as_ref()),
                    )
                    .expect("entry not found");

                AddressBook::load().remove(id);

                println!("Entry deleted from address book");
            }

            AddressBookViewActions::ShowQRCode { id, address, name } => {
                let (_, entry) = AddressBook::load()
                    .find(
                        &id.or(carry_on.id),
                        &address.or(carry_on.address),
                        &name.as_ref().or(carry_on.name.as_ref()),
                    )
                    .expect("entry not found");
            
                let qr = QrCode::new(entry.address.to_string()).expect("Failed to generate QR code");
                let qr_display = qr.render::<unicode::Dense1x2>().quiet_zone(false).build();
            
                println!("\n{}'s Address:\n{}\n", entry.name, entry.address);
                println!("QR Code:\n{}\n", qr_display);
            }
            
        }
    }
}
