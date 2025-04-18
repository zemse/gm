use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::tui::{
    events::Event,
    traits::{Component, HandleResult},
};

#[derive(Default)]
pub struct SetupPage;

impl Component for SetupPage {
    fn handle_event(&mut self, _event: &Event) -> HandleResult {
        HandleResult::default()
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer) -> Rect
    where
        Self: Sized,
    {
        "temp page".render(area, buf);
        area
    }
}
