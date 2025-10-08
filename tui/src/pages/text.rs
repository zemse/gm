use std::sync::mpsc::Sender;

use ratatui::{
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState, post_handle_event::PostHandleEventActions, traits::Component, AppEvent,
};

#[derive(Debug)]
pub struct TextPage {
    pub text: String,
    pub cursor: usize,
}

impl TextPage {
    pub fn new(text: String) -> Self {
        Self { text, cursor: 0 }
    }
}

impl Component for TextPage {
    fn handle_event(
        &mut self,
        _event: &AppEvent,
        _area: Rect,
        _popup_area: Rect,
        _transmitter: &Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        Ok(PostHandleEventActions::default())
    }

    fn render_component(
        &self,
        area: Rect,
        _popup_area: Rect,
        buf: &mut ratatui::prelude::Buffer,
        _shared_state: &crate::app::SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        Paragraph::new(Text::raw(&self.text))
            .wrap(Wrap { trim: false })
            .to_owned()
            .render(area, buf);

        area
    }
}
