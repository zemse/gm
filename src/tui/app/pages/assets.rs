use std::sync::{atomic::AtomicBool, mpsc, Arc};

use crossterm::event::KeyCode;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    tui::{
        app::{widgets::select::Select, Focus, SharedState},
        events::Event,
        traits::{Component, HandleResult},
    },
    utils::cursor::Cursor,
};

use super::{asset_transfer::AssetTransferPage, Page};

#[derive(Default)]
pub struct AssetsPage {
    cursor: Cursor,
}

impl Component for AssetsPage {
    fn handle_event(
        &mut self,
        event: &Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        self.cursor.handle(
            event,
            shared_state.assets.as_ref().map(|a| a.len()).unwrap_or(1),
        );

        let mut handle_result = HandleResult::default();

        #[allow(clippy::single_match)]
        match event {
            Event::Input(key_event) => match key_event.code {
                KeyCode::Enter =>
                {
                    #[allow(clippy::field_reassign_with_default)]
                    if let Some(assets) = shared_state.assets.as_ref() {
                        handle_result.page_inserts.push(Page::AssetTransfer(
                            AssetTransferPage::new(&assets[self.cursor.current]),
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
        if let Some(list) = shared_state.assets.as_ref() {
            if list.is_empty() {
                "no assets on the address".render(area, buf);
            } else {
                Select {
                    list,
                    cursor: &self.cursor,
                    focus: shared_state.focus == Focus::Main,
                    focus_style: None,
                }
                .render(area, buf);
            }
        } else {
            "loading assets...".render(area, buf);
        }

        area
    }
}
