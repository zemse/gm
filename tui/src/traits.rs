use std::sync::{atomic::AtomicBool, mpsc, Arc};

use gm_ratatui_extra::{act::Act, thematize::Thematize};
use ratatui::widgets::{Block, Widget};

use crate::app::SharedState;

use super::{events::Event, pages::Page};

// TODO change the name of this struct and trait, something like trait PostHandleEvent
#[derive(Default, Debug)]
pub struct Actions {
    // Number of pages to go back, usually 1. // TODO change to bool
    pub page_pops: usize,
    // Page to insert into the context stack.
    pub page_inserts: Vec<Page>,
    // Number of [ESC] key presses to ignore. This is to enable the current page
    // wants to handle the [ESC] key.
    pub ignore_esc: bool,
    // Regenerate the data for the current page, this is used when we expect
    // that the external state is updated and we need to reflect that in the UI.
    pub reload: bool,
    // Clears data for assets and refetches them.
    pub refresh_assets: bool,
}

impl Act for Actions {
    fn merge(&mut self, other: Actions) {
        self.page_pops += other.page_pops;
        self.page_inserts.extend(other.page_inserts);
        self.ignore_esc |= other.ignore_esc;
        self.reload |= other.reload;
        self.refresh_assets |= other.refresh_assets;
    }

    fn ignore_esc(&mut self) {
        self.ignore_esc = true;
    }
}

pub trait Component {
    fn reload(&mut self, _shared_state: &SharedState) -> crate::Result<()> {
        Ok(())
    }

    fn text_input_mut(&mut self) -> Option<&mut String> {
        None
    }

    async fn exit_threads(&mut self) {}

    fn set_focus(&mut self, _focus: bool) {}

    /// Handles an event and returns any actions to be performed.
    /// This cannot be async to prevent TUI render from blocking.
    /// `event` is mutable to allow taking ownership of inner data when needed.
    fn handle_event(
        &mut self,
        event: &Event,
        area: ratatui::prelude::Rect,
        transmitter: &mpsc::Sender<Event>,
        shutdown_signal: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> crate::Result<Actions>;

    /// Renders the component into the given area and returns the area that was
    /// actually used.
    // TODO do something about the return type here
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
        block
            .style(shared_state.theme.block())
            .border_type(shared_state.theme.border_type())
            .render(area, buf);
        self.render_component(inner_area, buf, shared_state);
        area
    }
}
