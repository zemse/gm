use std::sync::{atomic::AtomicBool, mpsc, Arc};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::tui::{
    app::SharedState,
    events::Event,
    traits::{Component, HandleResult},
};

#[derive(Default)]
pub struct TransactionPage;

impl Component for TransactionPage {
    fn handle_event(
        &mut self,
        _event: &Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        Ok(HandleResult::default())
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, _: &SharedState) -> Rect
    where
        Self: Sized,
    {
        "temp page".render(area, buf);
        area
    }
}
