use std::{
    io,
    sync::{atomic::AtomicBool, mpsc, Arc},
};

use alloy::primitives::Address;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use pages::{main_menu::MainMenuPage, Page};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Widget},
    DefaultTerminal,
};
use widgets::{footer::Footer, popup_ok::PopupOk, sidebar::Sidebar, title::Title};

use crate::disk::{Config, DiskInterface};

use super::{
    events::Event,
    traits::{BorderedWidget, Component},
};

pub mod pages;
pub mod widgets;

// Shared among all pages
// pub struct SharedState {
//     cursor_freeze: bool,
// }

pub struct App {
    pub exit: bool,
    pub current_account: Option<Address>,
    pub context: Vec<Page>,
    pub eth_price: Option<String>,
    pub testnet_mode: bool,
    pub fatal_error: Option<String>,
    // pub shared_state: SharedState,
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load();

        Self {
            exit: false,
            eth_price: None,
            testnet_mode: config.testnet_mode,
            current_account: config.current_account,
            fatal_error: None,
            // shared_state: SharedState {
            //     cursor_freeze: false,
            // },
            context: vec![Page::MainMenu(MainMenuPage::default())],
        }
    }
}

impl App {
    pub fn draw(&self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| {
            frame.render_widget(self, frame.area());
        })?;
        Ok(())
    }

    pub async fn exit_threads(&mut self) {
        for page in &mut self.context {
            page.exit_threads().await;
        }
    }

    pub fn reload(&mut self) {
        let config = Config::load();
        self.testnet_mode = config.testnet_mode;
    }

    pub fn handle_event(
        &mut self,
        event: super::events::Event,
        tr: &mpsc::Sender<Event>,
        sd: &Arc<AtomicBool>,
    ) -> crate::Result<()> {
        if let Some(page) = self.current_page_mut() {
            let result = page.handle_event(&event, tr, sd)?;
            for _ in 0..result.page_pops {
                self.context.pop();
            }
            if result.reload {
                self.reload();
                if let Some(page) = self.current_page_mut() {
                    page.reload();
                }
            }
            self.context.extend(result.page_inserts);
        }

        if self.context.is_empty() {
            self.exit = true;
        }

        match event {
            Event::Input(key_event) => {
                // check if we should exit on 'q' press
                if key_event.kind == KeyEventKind::Press {
                    #[allow(clippy::single_match)]
                    match key_event.code {
                        KeyCode::Char(char) => {
                            // TODO can we quit using q as well?
                            // if char == 'q' && self.navigation.text_input().is_none() {
                            //     self.exit = true;
                            // }
                            if char == 'c' && key_event.modifiers == KeyModifiers::CONTROL {
                                self.exit = true;
                            }
                            if char == 'e' && key_event.modifiers == KeyModifiers::CONTROL {
                                self.fatal_error = Some("test error".to_string());
                                // self.shared_state.cursor_freeze = true;
                            }
                        }
                        KeyCode::Esc => {
                            if self.fatal_error.is_some() {
                                self.fatal_error = None;
                                // self.shared_state.cursor_freeze = false;
                            } else {
                                self.context.pop();
                                if self.context.is_empty() {
                                    self.exit = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::EthPriceUpdate(eth_price) => {
                self.eth_price = Some(eth_price);
            }
            Event::AccountChange(address) => {
                self.current_account = Some(address);
            }
            _ => {}
        };

        Ok(())
    }

    fn current_page(&self) -> Option<&Page> {
        self.context.last()
    }

    fn current_page_mut(&mut self) -> Option<&mut Page> {
        self.context.last_mut()
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let vertical_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ]);
        let [title_area, body_area, footer_area] = vertical_layout.areas(area);

        if let Some(page) = self.current_page() {
            Title {
                current_account: self.current_account.as_ref(),
            }
            .render(title_area, buf);

            // Body render
            if page.is_full_screen() {
                page.render_component_with_block(body_area, buf, Block::bordered());
            } else {
                let horizontal_layout =
                    Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]);
                let [left_area, right_area] = horizontal_layout.areas(body_area);

                page.render_component_with_block(left_area, buf, Block::bordered());

                Sidebar {
                    eth_price: &self.eth_price,
                    testnet_mode: &self.testnet_mode,
                }
                .render_with_block(
                    right_area,
                    buf,
                    Block::bordered().border_type(BorderType::Plain),
                );
            }

            Footer {
                exit: &self.exit,
                is_main_menu: &page.is_main_menu(),
            }
            .render(footer_area, buf);
        }

        if let Some(fatal_error) = &self.fatal_error {
            PopupOk {
                title: "Fatal Error",
                message: fatal_error,
            }
            .render(area, buf);
        };
    }
}
