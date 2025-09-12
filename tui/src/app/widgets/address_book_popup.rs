use crate::app::pages::address_book::AddressBookMenuItem;

use super::filter_select_popup::FilterSelectPopup;

pub struct AddressBookPopup {
    inner: FilterSelectPopup<AddressBookMenuItem>,
}

impl Default for AddressBookPopup {
    fn default() -> Self {
        Self {
            inner: FilterSelectPopup::new(
                "Address Book",
                Some("No items in your address book, feel free to add from the main menu."),
            ),
        }
    }
}

use std::ops::{Deref, DerefMut};

impl Deref for AddressBookPopup {
    type Target = FilterSelectPopup<AddressBookMenuItem>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AddressBookPopup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
