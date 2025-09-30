use gm_ratatui_extra::select_owned::SelectOwned;
use gm_utils::disk_storage::DiskStorageInterface;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::fmt::Display;
use std::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::app::SharedState;
use crate::pages::network_create::NetworkCreatePage;
use crate::pages::Page;
use crate::traits::{Actions, Component};
use crate::AppEvent;
use gm_utils::network::{Network, NetworkStore};

#[derive(Debug, PartialEq)]
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
    select: SelectOwned<NetworkSelect>,
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
            select: SelectOwned::new(Some(list)),
        })
    }
}
impl Component for NetworkPage {
    fn set_focus(&mut self, focus: bool) {
        self.select.focus = focus;
    }

    fn reload(&mut self, _ss: &SharedState) -> crate::Result<()> {
        let fresh = Self::new()?;
        self.select = fresh.select;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let mut result = Actions::default();

        self.select
            .handle_event(event.input_event(), area, |item| {
                let network_store = NetworkStore::load()?;
                match item {
                    NetworkSelect::Create => {
                        let network_index = network_store.networks.len();
                        result
                            .page_inserts
                            .push(Page::NetworkCreate(NetworkCreatePage::new(
                                network_index,
                                Network::default(),
                            )?));
                        result.reload = true;
                    }

                    NetworkSelect::Existing(name) => {
                        let network_index = network_store
                            .networks
                            .iter()
                            .position(|n| n.name == name.name)
                            .unwrap();
                        result
                            .page_inserts
                            .push(Page::NetworkCreate(NetworkCreatePage::new(
                                network_index,
                                *name.clone(),
                            )?));
                        result.reload = true;
                    }
                }
                Ok::<(), crate::Error>(())
            })?;

        Ok(result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        self.select.render(area, buf, &shared_state.theme);
        area
    }
}
