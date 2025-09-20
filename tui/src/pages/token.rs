use crate::app::SharedState;
use crate::pages::network_create::NetworkCreatePage;
use crate::pages::token_create::TokenCreatePage;
use crate::pages::Page;
use crate::traits::{Actions, Component};
use crate::Event;
use gm_ratatui_extra::act::Act;
use gm_ratatui_extra::cursor::Cursor;
use gm_ratatui_extra::select::Select;
use gm_ratatui_extra::thematize::Thematize;
use gm_utils::network::{Network, Token};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use std::fmt::{Display, Formatter};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::Arc;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct TokenPage {
    cursor: Cursor,
    focus: bool,
    list: Vec<TokenSelect>,
    network: Network,
    network_index: usize,
}
impl TokenPage {
    pub fn new(network_index: usize, network: Network) -> crate::Result<Self> {
        let mut list = vec![TokenSelect::Create];
        let tokens = network.tokens.clone();
        list.extend(
            tokens
                .into_iter()
                .map(|t| TokenSelect::Existing(Box::new(t)))
                .collect::<Vec<_>>(),
        );
        Ok(Self {
            cursor: Cursor::default(),
            focus: true,
            list,
            network,
            network_index,
        })
    }
}
impl Component for TokenPage {
    fn reload(&mut self, _ss: &SharedState) -> crate::Result<()> {
        let fresh = Self::new(self.network_index, self.network.clone())?;
        self.list = fresh.list;
        Ok(())
    }

    fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        _transmitter: &Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let cursor_max = self.list.len();
        self.cursor.handle(event.key_event(), cursor_max);

        let mut handle_result = Actions::default();
        handle_result.ignore_esc();
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                #[allow(clippy::single_match)]
                match key_event.code {
                    KeyCode::Enter => match &self.list[self.cursor.current] {
                        TokenSelect::Create => {
                            let token_index = self.network.tokens.len();
                            handle_result.page_pops = 1;
                            handle_result.page_inserts.push(Page::TokenCreate(
                                TokenCreatePage::new(
                                    token_index,
                                    self.network_index,
                                    self.network.clone(),
                                )?,
                            ));
                            handle_result.reload = true;
                        }

                        TokenSelect::Existing(token) => {
                            let token_index = self
                                .network
                                .tokens
                                .iter()
                                .position(|t| t.contract_address == token.contract_address)
                                .unwrap();
                            handle_result.page_pops = 1;
                            handle_result.page_inserts.push(Page::TokenCreate(
                                TokenCreatePage::new(
                                    token_index,
                                    self.network_index,
                                    self.network.clone(),
                                )?,
                            ));
                            handle_result.reload = true;
                        }
                    },
                    KeyCode::Esc => {
                        handle_result.page_pops = 1;
                        handle_result.page_inserts.push(Page::NetworkCreate(
                            NetworkCreatePage::new(self.network_index, self.network.clone())?,
                        ));
                    }
                    _ => {}
                }
            }
        };
        Ok(handle_result)
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
