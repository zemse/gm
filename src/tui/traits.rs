use std::sync::{atomic::AtomicBool, mpsc, Arc};

use ratatui::{
    layout::Rect,
    widgets::{Block, Widget},
};

use super::{
    app::{pages::Page, SharedState},
    events::Event,
};

pub trait BorderedWidget {
    fn render_with_block(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        block: Block<'_>,
    ) where
        Self: Sized;
}

impl<T: Widget> BorderedWidget for T {
    fn render_with_block(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        block: Block<'_>,
    ) where
        Self: Sized,
    {
        let inner_area = block.inner(area);
        block.render(area, buf);
        self.render(inner_area, buf);
    }
}

pub trait WidgetHeight {
    fn height_used(&self, area: ratatui::prelude::Rect) -> u16;
}

#[derive(Default)]
pub struct HandleResult {
    pub page_pops: usize,
    pub page_inserts: Vec<Page>,
    pub reload: bool,
}

pub trait Component {
    fn reload(&mut self) {}

    fn text_input_mut(&mut self) -> Option<&mut String> {
        None
    }

    async fn exit_threads(&mut self) {}

    fn handle_event(
        &mut self,
        event: &Event,
        transmitter: &mpsc::Sender<Event>,
        shutdown_signal: &Arc<AtomicBool>,
    ) -> crate::Result<HandleResult>;

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized;

    fn render_component_with_block(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        block: Block<'_>,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        let inner_area = block.inner(area);
        block.render(area, buf);
        self.render_component(inner_area, buf, shared_state);
        area
    }
}
