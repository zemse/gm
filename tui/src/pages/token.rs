use crate::app::SharedState;
use crate::pages::token_create::TokenCreatePage;
use crate::pages::Page;
use crate::post_handle_event::PostHandleEventActions;
use crate::traits::Component;
use crate::AppEvent;
use gm_ratatui_extra::act::Act;
use gm_ratatui_extra::extensions::ThemedWidget;
use gm_ratatui_extra::select::{Select, SelectEvent};
use gm_utils::network::{Network, Token};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::fmt::{Display, Formatter};
use std::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

#[derive(Debug, PartialEq)]
enum TokenSelect {
    Create,
    Existing(Box<Token>),
}

impl Display for TokenSelect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenSelect::Create => write!(f, "Create new token"),
            TokenSelect::Existing(token) => write!(f, "{}", token.name),
        }
    }
}

impl TokenSelect {
    fn get_list(network: &Network) -> crate::Result<Vec<TokenSelect>> {
        let mut list = vec![TokenSelect::Create];
        list.extend(
            network
                .tokens
                .iter()
                .cloned()
                .map(|t| TokenSelect::Existing(Box::new(t)))
                .collect::<Vec<_>>(),
        );
        Ok(list)
    }
}

#[derive(Debug)]
pub struct TokenPage {
    select: Select<TokenSelect>,
    network: Network,
    network_index: usize,
}
impl TokenPage {
    pub fn new(network_index: usize, network: Network) -> crate::Result<Self> {
        Ok(Self {
            select: Select::default().with_list(TokenSelect::get_list(&network)?),
            network,
            network_index,
        })
    }
}
impl Component for TokenPage {
    fn reload(&mut self, shared_state: &SharedState) -> crate::Result<()> {
        let network = shared_state
            .networks
            .get_by_name(&self.network.name)
            .ok_or(crate::Error::NetworkNotFound(self.network.name.clone()))?;

        self.select
            .update_list(Some(TokenSelect::get_list(&network)?));

        Ok(())
    }

    fn set_focus(&mut self, focus: bool) {
        self.select.set_focus(focus);
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _popup_area: Rect,
        _transmitter: &Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let mut handle_result = PostHandleEventActions::default();

        if let Some(SelectEvent::Select(item)) =
            self.select.handle_event(event.input_event(), area)?
        {
            match item {
                TokenSelect::Create => {
                    let token_index = self.network.tokens.len();
                    handle_result.page_insert(Page::TokenCreate(TokenCreatePage::new(
                        true,
                        token_index,
                        self.network_index,
                        self.network.clone(),
                    )?));
                }
                TokenSelect::Existing(token) => {
                    let token_index = self
                        .network
                        .tokens
                        .iter()
                        .position(|t| t.contract_address == token.contract_address)
                        .unwrap();
                    handle_result.page_insert(Page::TokenCreate(TokenCreatePage::new(
                        false,
                        token_index,
                        self.network_index,
                        self.network.clone(),
                    )?));
                }
            }
        }

        handle_result.ignore_esc();

        Ok(handle_result)
    }

    fn render_component(
        &self,
        area: Rect,
        _popup_area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        self.select.render(area, buf, &shared_state.theme);
        area
    }
}
