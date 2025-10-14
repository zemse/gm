use std::sync::mpsc;

use gm_ratatui_extra::{
    extensions::ThemedWidget,
    select::{Select, SelectEvent},
};
use gm_utils::assets::Asset;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState, post_handle_event::PostHandleEventActions, traits::Component, AppEvent,
};

use super::{asset_transfer::AssetTransferPage, Page};

#[derive(Debug)]
pub struct AssetsPage {
    select: Select<Asset>,
}

impl AssetsPage {
    pub fn new(assets: Option<Vec<Asset>>) -> crate::Result<Self> {
        Ok(Self {
            select: Select::new(assets, false)
                .with_loading_text("Loading assets...")
                .with_empty_text("No assets found in portfolio"),
        })
    }
}

impl Component for AssetsPage {
    fn set_focus(&mut self, focus: bool) {
        self.select.set_focus(focus);
    }

    fn set_cursor(&mut self, cursor: usize) {
        // self.select.cursor.current = cursor.min(self.select.len().saturating_sub(1));
        self.select.set_cursor(cursor);
    }

    fn get_cursor(&self) -> Option<usize> {
        Some(self.select.cursor())
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _popup_area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let assets = shared_state.assets_read()?;

        if let AppEvent::AssetsUpdate(_, _) = event {
            self.select.update_list(assets);
        }

        let mut handle_result = PostHandleEventActions::default();
        if let Some(SelectEvent::Select(asset)) =
            self.select.handle_event(event.input_event(), area)?
        {
            handle_result.page_insert(Page::AssetTransfer(AssetTransferPage::new(asset)?));
        }

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
        if self.select.list_is_none() && shared_state.online == Some(false) {
            "need internet access to fetch the portfolio".render(area, buf);
        } else {
            self.select.render(area, buf, &shared_state.theme);
        }

        area
    }
}
