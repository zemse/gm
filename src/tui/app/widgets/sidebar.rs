use std::sync::{atomic::AtomicBool, mpsc, Arc};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    disk::{Config, DiskInterface},
    tui::{
        app::{
            pages::{trade::TradePage, Page},
            Focus, SharedState,
        },
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
        shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        self.cursor.handle(event, 2);

        let mut result = HandleResult::default();

        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                #[allow(clippy::single_match)]
                match key_event.code {
                    KeyCode::Enter => match self.cursor.current {
                        0 => result.page_inserts.push(Page::Trade(TradePage::default())),
                        1 => {
                            let mut config = Config::load();
                            config.testnet_mode = !shared_state.testnet_mode;
                            config.save();
                            result.reload = true;
                            result.refresh_assets = true;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        Ok(result)
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

        let portfolio = if let Some(assets) = &shared_state.assets {
            let portfolio = assets
                .iter()
                .fold(0.0, |acc, asset| acc + asset.usd_value().unwrap_or(0.0));

            format!("Portfolio: ${portfolio}")
        } else {
            "Portfolio: Loading...".to_string()
        };

        Select {
            list: &vec![
                format!("EthPrice: {eth_price}"),
                format!("Testnet Mode: {}", shared_state.testnet_mode),
                portfolio,
            ],
            cursor: &self.cursor,
            focus: shared_state.focus == Focus::Sidebar,
        }
        .render(area, buf);

        area
    }
}
