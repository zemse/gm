use crossterm::event::{KeyCode, KeyEventKind};
use gm_utils::disk::DiskInterface;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use std::fmt::Display;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};

use crate::app::pages::network_create::NetworkCreatePage;
use crate::app::pages::Page;
use crate::app::widgets::{cursor::Cursor, select::Select};
use crate::app::SharedState;
use crate::traits::{Component, HandleResult};
use crate::Event;
use gm_utils::network::{Network, NetworkStore};

enum NetworkSelect {
    Create,
    Existing(Box<Network>),
}
impl Display for NetworkSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkSelect::Create => write!(f, "Create new network"),
            NetworkSelect::Existing(name) => write!(f, "{name}"),
        }
    }
}
pub struct NetworkPage {
    cursor: Cursor,
    focus: bool,
    list: Vec<NetworkSelect>,
}
impl NetworkPage {
    pub fn new() -> crate::Result<Self> {
        let mut list = vec![NetworkSelect::Create];
        list.extend(
            NetworkStore::load()?
                .networks
                .into_iter()
                .map(|network| NetworkSelect::Existing(Box::new(network)))
                .collect::<Vec<_>>(),
        );
        Ok(Self {
            cursor: Cursor::default(),
            focus: true,
            list,
        })
    }
}
impl Component for NetworkPage {
    fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    fn reload(&mut self, _ss: &SharedState) -> crate::Result<()> {
        let fresh = Self::new()?;
        self.list = fresh.list;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        let cursor_max = self.list.len();
        self.cursor.handle(event, cursor_max);

        let mut result = HandleResult::default();
        let network_store = NetworkStore::load()?;
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                #[allow(clippy::single_match)]
                match key_event.code {
                    KeyCode::Enter => {
                        match &self.list[self.cursor.current] {
                            NetworkSelect::Create => {
                                let network_index = network_store.networks.len();
                                result.page_inserts.push(Page::NetworkCreate(
                                    NetworkCreatePage::new(network_index, Network::default())?,
                                ));
                                result.reload = true;
                            }

                            NetworkSelect::Existing(name) => {
                                let network_index = network_store
                                    .networks
                                    .iter()
                                    .position(|n| n.name == name.name)
                                    .unwrap();
                                result.page_inserts.push(Page::NetworkCreate(
                                    NetworkCreatePage::new(network_index, *name.clone())?,
                                ));
                                result.reload = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        };

        Ok(result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        Select {
            list: &self.list,
            cursor: &self.cursor,
            focus: self.focus,
            focus_style: shared_state.theme.select(),
        }
        .render(area, buf);
        area
    }
}
