use std::sync::mpsc::Sender;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState, post_handle_event::PostHandleEventActions, traits::Component, AppEvent,
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
        _popup_area: Rect,
        _transmitter: &Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        if let AppEvent::Input(key_event) = event {
            self.data = Some(format!("{key_event:?}"))
        }

        Ok(PostHandleEventActions::default())
    }

    fn render_component(
        &self,
        area: Rect,
        _popup_area: Rect,
        buf: &mut Buffer,
        _shared_state: &SharedState,
    ) -> Rect
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
