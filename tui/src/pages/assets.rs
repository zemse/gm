use std::sync::{atomic::AtomicBool, mpsc, Arc};

use gm_ratatui_extra::{cursor::Cursor, select::Select, thematize::Thematize};
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, layout::Rect, widgets::Widget};

use crate::{
    app::SharedState,
    events::Event,
    traits::{Actions, Component},
};

use super::{asset_transfer::AssetTransferPage, Page};

#[derive(Default, Debug)]
pub struct AssetsPage {
    cursor: Cursor,
    focus: bool,
}

impl Component for AssetsPage {
    fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let assets = shared_state.assets_read()?;

        self.cursor.handle(
            event.key_event(),
            assets.as_ref().map(|a| a.len()).unwrap_or(1),
        );

        let mut handle_result = Actions::default();

        #[allow(clippy::single_match)]
        match event {
            Event::Input(key_event) => match key_event.code {
                KeyCode::Enter =>
                {
                    #[allow(clippy::field_reassign_with_default)]
                    if let Some(assets) = assets.as_ref() {
                        handle_result.page_inserts.push(Page::AssetTransfer(
                            AssetTransferPage::new(&assets[self.cursor.current])?,
                        ));
                    }
                }

                _ => {}
            },
            _ => {}
        }

        Ok(handle_result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        if let Some(list) = shared_state.assets_read().ok().flatten().as_ref() {
            if list.is_empty() {
                "no assets on the address".render(area, buf);
            } else {
                Select {
                    list,
                    cursor: &self.cursor,
                    focus: self.focus,
                    focus_style: shared_state.theme.select_focused(),
                }
                .render(area, buf);
            }
        } else {
            "loading assets...".render(area, buf);
        }

        area
    }
}
