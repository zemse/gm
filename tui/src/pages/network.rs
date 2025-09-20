use gm_ratatui_extra::cursor::Cursor;
use gm_ratatui_extra::select::Select;
use gm_ratatui_extra::thematize::Thematize;
use gm_utils::disk_storage::DiskStorageInterface;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use std::fmt::Display;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};

use crate::app::SharedState;
use crate::pages::network_create::NetworkCreatePage;
use crate::pages::Page;
use crate::traits::{Actions, Component};
use crate::Event;
use gm_utils::network::{Network, NetworkStore};

#[derive(Debug)]
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

#[derive(Debug)]
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
    ) -> crate::Result<Actions> {
        let cursor_max = self.list.len();
        self.cursor.handle(event.key_event(), cursor_max);

        let mut result = Actions::default();
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
            focus_style: shared_state.theme.select_focused(),
        }
        .render(area, buf);
        area
    }
}
