use std::sync::{atomic::AtomicBool, mpsc, Arc};

use ratatui::widgets::{Block, Widget};

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
    // Number of pages to go back, usually 1.
    pub page_pops: usize,
    // Page to insert into the context stack.
    pub page_inserts: Vec<Page>,
    // Number of [ESC] key presses to ignore. This is to enable the current page
    // wants to handle the [ESC] key.
    pub esc_ignores: usize,
    // Regenerate the data for the current page, this is used when we expect
    // that the external state is updated and we need to reflect that in the UI.
    pub reload: bool,
    // Clears data for assets and refetches them.
    pub refresh_assets: bool,
}

pub trait Component {
    // TODO rename to `reload` or `refresh_component`
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
        shared_state: &SharedState,
    ) -> crate::Result<HandleResult>;

    // Renders the component into the given area and returns the area that was
    // actually used.
    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized;

    fn render_component_with_block(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        block: Block<'_>,
        shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        let inner_area = block.inner(area);
        block.render(area, buf);
        self.render_component(inner_area, buf, shared_state);
        area
    }
}
