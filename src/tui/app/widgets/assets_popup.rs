use crate::utils::assets::Asset;

use super::filter_select_popup::FilterSelectPopup;

pub struct AssetsPopup {
    inner: FilterSelectPopup<Asset>,
}

impl Default for AssetsPopup {
    fn default() -> Self {
        Self {
            inner: FilterSelectPopup::new(
                "Assets",
                Some("No assets available. Please fund your account."),
            ),
        }
    }
}

use std::ops::{Deref, DerefMut};

impl Deref for AssetsPopup {
    type Target = FilterSelectPopup<Asset>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AssetsPopup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
