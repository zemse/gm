use std::sync::{atomic::AtomicBool, mpsc::Sender, Arc};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::tui::{
    app::SharedState,
    traits::{Component, HandleResult, RectUtil},
    Event,
};

pub struct FusionPlusPage {}

impl Component for FusionPlusPage {
    fn handle_event(
        &mut self,
        event: &Event,
        area: Rect,
        transmitter: &Sender<Event>,
        shutdown_signal: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        Ok(HandleResult::default())
    }

    fn render_component(
        &self,
        area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        if let Ok(remaining_area) = area.consume_height(1) {
            "temp".render(area, buf);
            remaining_area
        } else {
            area
        }
    }
}
