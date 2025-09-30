use std::sync::mpsc::Sender;

use ratatui::{layout::Rect, widgets::Widget};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState,
    traits::{Actions, Component},
    AppEvent,
};

#[derive(Default, Debug)]
pub struct DevKeyCapturePage {
    data: Option<String>,
}

impl Component for DevKeyCapturePage {
    fn handle_event(
        &mut self,
        event: &AppEvent,
        _area: Rect,
        _transmitter: &Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        if let AppEvent::Input(key_event) = event {
            self.data = Some(format!("{key_event:?}"))
        }

        Ok(Actions::default())
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
