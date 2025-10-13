use gm_ratatui_extra::filter_select_popup::FilterSelectPopup;
use gm_utils::{assets::Asset, network::Network};

use crate::pages::address_book::AddressBookMenuItem;

pub type AddressBookPopup = FilterSelectPopup<AddressBookMenuItem>;
pub fn address_book_popup() -> AddressBookPopup {
    FilterSelectPopup::new("Address Book")
        .with_empty_text("No items in your address book, feel free to add from the main menu.")
}

pub type AssetsPopup = FilterSelectPopup<Asset>;
pub fn assets_popup() -> AssetsPopup {
    FilterSelectPopup::new("Assets")
        .with_empty_text("No assets available. Please fund your account.")
}

pub type NetworksPopup = FilterSelectPopup<Network>;
pub fn networks_popup() -> NetworksPopup {
    FilterSelectPopup::new("Networks").with_empty_text("No networks available. It's weird. Please check your configuration or create github issue.")
}
