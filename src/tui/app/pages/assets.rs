use std::sync::{atomic::AtomicBool, mpsc, Arc};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::tui::{
    events::Event,
    traits::{Component, HandleResult},
};

#[derive(Default)]
pub struct AssetsPage;

impl Component for AssetsPage {
    fn handle_event(
        &mut self,
        _event: &Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
    ) -> crate::Result<HandleResult> {
        Ok(HandleResult::default())
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer) -> Rect
    where
        Self: Sized,
    {
        "assets page".render(area, buf);
        area
    }
}
