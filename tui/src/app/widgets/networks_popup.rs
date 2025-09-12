use gm_utils::network::Network;

use super::filter_select_popup::FilterSelectPopup;

pub struct NetworksPopup {
    inner: FilterSelectPopup<Network>,
}

impl Default for NetworksPopup {
    fn default() -> Self {
        Self {
            inner: FilterSelectPopup::new(
                "Networks",
                Some("No networks available. It's weird. Please check your configuration or create github issue."),
            ),
        }
    }
}

use std::ops::{Deref, DerefMut};

impl Deref for NetworksPopup {
    type Target = FilterSelectPopup<Network>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for NetworksPopup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
