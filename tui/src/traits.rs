use std::sync::mpsc;

use gm_ratatui_extra::{act::Act, thematize::Thematize};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Widget},
};
use tokio_util::sync::CancellationToken;

use crate::{app::SharedState, AppEvent};

use super::pages::Page;

// TODO change the name of this struct and trait, something like trait PostHandleEvent
#[derive(Default, Debug)]
pub struct Actions {
    // Remove the current page from the context stack.
    pub page_pop: bool,
    // Remove all pages from the context stack.
    pub page_pop_all: bool,
    // Page to insert into the context stack.
    pub page_inserts: Vec<Page>,
    // Enable if current page wants to handle the [ESC] key.
    pub ignore_esc: bool,
    // Ignore [CTRL+C] key presses which would quit app
    pub ignore_ctrlc: bool,
    // Ignore left arrow key presses which would move focus to menu
    pub ignore_left: bool,
    // Regenerate the data for the current page, this is used when we expect
    // that the external state is updated and we need to reflect that in the UI.
    pub reload: bool,
    // Clears data for assets and refetches them.
    pub refresh_assets: bool,
}

impl Act for Actions {
    fn merge(&mut self, other: Actions) {
        self.page_pop |= other.page_pop;
        self.page_pop_all |= other.page_pop_all;
        self.page_inserts.extend(other.page_inserts);
        self.ignore_esc |= other.ignore_esc;
        self.reload |= other.reload;
        self.refresh_assets |= other.refresh_assets;
    }

    fn ignore_esc(&mut self) {
        self.ignore_esc = true;
    }
}

impl Actions {
    pub fn ignore_ctrlc(&mut self) {
        self.ignore_ctrlc = true;
    }
}

pub trait Component {
    fn reload(&mut self, _shared_state: &SharedState) -> crate::Result<()> {
        Ok(())
    }

    async fn exit_threads(&mut self) {}

    fn set_focus(&mut self, _focus: bool) {}

    fn set_cursor(&mut self, _cursor: usize) {}

    fn get_cursor(&self) -> Option<usize> {
        None
    }

    /// Handles an event and returns any actions to be performed.
    /// This cannot be async to prevent TUI render from blocking.
    /// `event` is mutable to allow taking ownership of inner data when needed.
    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        transmitter: &mpsc::Sender<AppEvent>,
        shutdown_signal: &CancellationToken,
        shared_state: &SharedState,
    ) -> crate::Result<Actions>;

    /// Renders the component into the given area and returns the area that was
    /// actually used.
    // TODO do something about the return type here
    fn render_component(
        &self,
        area: Rect,
        popup_area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized;

    #[allow(dead_code)]
    fn render_component_with_block(
        &self,
        area: Rect,
        popup_area: Rect,
        buf: &mut Buffer,
        block: Block<'_>,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        let inner_area = block.inner(area);
        block
            .style(shared_state.theme.style())
            .border_type(shared_state.theme.border_type())
            .render(area, buf);
        self.render_component(inner_area, popup_area, buf, shared_state);
        area
    }
}
