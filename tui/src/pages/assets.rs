use std::sync::mpsc;

use gm_ratatui_extra::{extensions::ThemedWidget, select_owned::SelectOwned};
use gm_utils::assets::Asset;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState,
    traits::{Actions, Component},
    AppEvent,
};

use super::{asset_transfer::AssetTransferPage, Page};

#[derive(Debug)]
pub struct AssetsPage {
    select: SelectOwned<Asset>,
}

impl AssetsPage {
    pub fn new(assets: Option<Vec<Asset>>) -> crate::Result<Self> {
        Ok(Self {
            select: SelectOwned::new(assets, false),
        })
    }
}

impl Component for AssetsPage {
    fn set_focus(&mut self, focus: bool) {
        self.select.focus = focus;
    }

    fn set_cursor(&mut self, cursor: usize) {
        self.select.cursor.current = cursor.min(self.select.len().saturating_sub(1));
    }

    fn get_cursor(&self) -> Option<usize> {
        Some(self.select.cursor.current)
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let assets = shared_state.assets_read()?;

        if let AppEvent::AssetsUpdate(_, _) = event {
            self.select.update_list(assets);
        }

        let mut handle_result = Actions::default();
        self.select.handle_event(
            event.input_event(),
            area,
            |asset| {
                handle_result
                    .page_inserts
                    .push(Page::AssetTransfer(AssetTransferPage::new(asset)?));
                Ok::<(), crate::Error>(())
            },
            |_| Ok(()),
        )?;

        Ok(handle_result)
    }

    fn render_component(
        &self,
        area: Rect,
        _popup_area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        if let Some(list) = self.select.list.as_ref() {
            if list.is_empty() {
                "no assets on the address".render(area, buf);
            } else {
                self.select.render(area, buf, &shared_state.theme);
            }
        } else if shared_state.online == Some(false) {
            "need internet access to fetch the portfolio".render(area, buf);
        } else {
            "loading assets...".render(area, buf);
        }

        area
    }
}
