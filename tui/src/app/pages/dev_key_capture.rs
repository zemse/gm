use std::sync::{atomic::AtomicBool, mpsc::Sender, Arc};

use ratatui::{layout::Rect, widgets::Widget};

use crate::{
    app::SharedState,
    traits::{Component, HandleResult},
    Event,
};

#[derive(Default)]
pub struct DevKeyCapturePage {
    data: Option<String>,
}

impl Component for DevKeyCapturePage {
    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        _transmitter: &Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        if let Event::Input(key_event) = event {
            self.data = Some(format!("{key_event:?}"))
        }

        Ok(HandleResult::default())
    }

    fn render_component(
        &self,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
        _shared_state: &crate::app::SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        if let Some(data) = self.data.as_ref() {
            data.render(area, buf);
        } else {
            "Press any key to capture the event".render(area, buf);
        }

        area
    }
}
