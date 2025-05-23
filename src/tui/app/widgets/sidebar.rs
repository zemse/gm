use std::sync::{atomic::AtomicBool, mpsc, Arc};

use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::Line, widgets::Widget};

use crate::{
    tui::{
        app::{Focus, SharedState},
        traits::{Component, HandleResult},
        Event,
    },
    utils::cursor::Cursor,
};

use super::select::Select;

#[derive(Default)]
pub struct Sidebar {
    pub focus: bool,
    pub cursor: Cursor,
}

impl Component for Sidebar {
    fn handle_event(
        &mut self,
        event: &crate::tui::Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        self.cursor.handle(event, 2);

        Ok(HandleResult::default())
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        let eth_price = if let Some(eth_price) = &shared_state.eth_price {
            eth_price.clone()
        } else {
            match shared_state.online {
                Some(true) | None => "Loading...".to_string(),
                Some(false) => "Unable to fetch".to_string(),
            }
        };

        Select {
            list: &vec![
                Line::from(format!("EthPrice: {eth_price}")).bold(),
                Line::from(format!("Testnet Mode: {}", shared_state.testnet_mode)).bold(),
            ],
            cursor: &self.cursor,
            focus: shared_state.focus == Focus::Sidebar,
        }
        .render(area, buf);

        area
    }
}
