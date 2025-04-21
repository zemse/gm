use std::sync::mpsc;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::tui::{
    events::Event,
    traits::{Component, HandleResult},
};

#[derive(Default)]
pub struct AccountCreatePage;

impl Component for AccountCreatePage {
    fn handle_event(
        &mut self,
        _event: &Event,
        _transmitter: &mpsc::Sender<Event>,
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
