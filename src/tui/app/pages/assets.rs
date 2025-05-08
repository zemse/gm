use std::sync::{atomic::AtomicBool, mpsc, Arc};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    tui::{
        app::{widgets::select::Select, SharedState},
        events::Event,
        traits::{Component, HandleResult},
    },
    utils::cursor::Cursor,
};

#[derive(Default)]
pub struct AssetsPage {
    cursor: Cursor,
}

impl Component for AssetsPage {
    fn handle_event(
        &mut self,
        _event: &Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
    ) -> crate::Result<HandleResult> {
        Ok(HandleResult::default())
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
                }
                .render(area, buf);
            }
        } else {
            "loading assets...".render(area, buf);
        }

        area
    }
}
