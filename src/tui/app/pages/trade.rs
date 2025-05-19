use ratatui::widgets::Widget;

use crate::tui::{
    app::widgets::candle_chart::CandleChart,
    traits::{Component, HandleResult},
};

pub struct TradePage;

impl Component for TradePage {
    fn handle_event(
        &mut self,
        _event: &crate::tui::Event,
        _transmitter: &std::sync::mpsc::Sender<crate::tui::Event>,
        _shutdown_signal: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> crate::Result<crate::tui::traits::HandleResult> {
        Ok(HandleResult::default())
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        _shared_state: &crate::tui::app::SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        CandleChart.render(area, buf);

        area
    }
}
