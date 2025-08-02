use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use crate::disk::DiskInterface;
use crate::network::{Network, NetworkStore, Token};
use crate::tui::app::pages::network::NetworkPage;
use crate::tui::app::pages::network_create::FormItem::TokensButton;
use crate::tui::app::pages::network_create::NetworkCreatePage;
use crate::tui::app::pages::Page;
use crate::tui::app::pages::token_create::TokenCreatePage;
use crate::tui::app::SharedState;
use crate::tui::app::widgets::select::Select;
use crate::tui::Event;
use crate::tui::traits::{Component, HandleResult};
use crate::utils::cursor::Cursor;

enum TokenSelect {
    Create,
    Existing(Box<Token>)
}

impl Display for TokenSelect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenSelect::Create => write!(f, "Create new token"),
            TokenSelect::Existing(token) => write!(f, "{}", token.name)
        }
    }
}
pub struct TokenPage {
    cursor: Cursor,
    focus: bool,
    list: Vec<TokenSelect>,
    network: Network,
    network_index: usize
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
            network_index
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

    fn handle_event(&mut self, event: &Event, area: Rect, transmitter: &Sender<Event>, shutdown_signal: &Arc<AtomicBool>, shared_state: &SharedState) -> crate::Result<HandleResult> {
        let cursor_max = self.list.len();
        self.cursor.handle(event, cursor_max);

        let mut handle_result = HandleResult::default();
        handle_result.esc_ignores = 1;
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                #[allow(clippy::single_match)]
                match key_event.code {
                    KeyCode::Enter => {
                        match &self.list[self.cursor.current] {
                           TokenSelect::Create => {
                               let token_index = self.network.tokens.len();
                               handle_result.page_pops = 1;
                                handle_result.page_inserts.push(Page::TokenCreate(
                                   TokenCreatePage::new(token_index, self.network_index, self.network.clone())?
                                ));
                               handle_result.reload = true;
                            }

                            TokenSelect::Existing(token) => {
                                let token_index = self.network.tokens.iter().position(|t| t.contract_address == token.contract_address).unwrap();
                                handle_result.page_pops = 1;
                                handle_result.page_inserts.push(Page::TokenCreate(
                                    TokenCreatePage::new(token_index, self.network_index, self.network.clone())?,
                                ));
                                handle_result.reload = true;
                            }
                        }
                    }
                    KeyCode::Esc => {
                        handle_result.page_pops = 1;
                        handle_result.page_inserts.push(Page::NetworkCreate(
                            NetworkCreatePage::new(self.network_index, self.network.clone())?
                        ));
                    },
                    _ => {}
                }
            }
        };
        Ok(handle_result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized
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