use std::{borrow::Cow, sync::mpsc};

use gm_ratatui_extra::thematize::Thematize;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Widget},
};
use tokio_util::sync::CancellationToken;

use crate::{app::SharedState, post_handle_event::PostHandleEventActions, AppEvent};

pub trait Component {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("")
    }

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
        popup_area: Rect,
        transmitter: &mpsc::Sender<AppEvent>,
        shutdown_signal: &CancellationToken,
        shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions>;

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
