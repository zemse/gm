use std::sync::{atomic::AtomicBool, mpsc::Sender, Arc};

use ratatui::{
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
};

use crate::{
    app::SharedState,
    traits::{Actions, Component},
    Event,
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
        _event: &Event,
        _area: Rect,
        _transmitter: &Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
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
        Paragraph::new(Text::raw(&self.text))
            .wrap(Wrap { trim: false })
            .to_owned()
            .render(area, buf);

        area
    }
}
