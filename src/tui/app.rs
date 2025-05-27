use std::{
    io,
    sync::{atomic::AtomicBool, mpsc, Arc},
};

use alloy::primitives::Address;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use pages::{main_menu::MainMenuPage, trade::TradePage, Page};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Color,
    text::Text,
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
    DefaultTerminal,
};
use widgets::{footer::Footer, popup::Popup, sidebar::Sidebar, title::Title};

use crate::{
    disk::{Config, DiskInterface},
    error::FmtError,
    utils::assets::Asset,
};

use super::{
    events::{self, Event},
    traits::{BorderedWidget, Component, HandleResult},
};

pub mod pages;
pub mod widgets;

#[derive(PartialEq)]
pub enum Focus {
    Main,
    Sidebar,
}

pub struct SharedState {
    pub online: Option<bool>,
    pub assets: Option<Vec<Asset>>,
    pub testnet_mode: bool,
    pub current_account: Option<Address>,
    pub eth_price: Option<String>,
    pub focus: Focus,
}

pub struct App {
    pub context: Vec<Page>,
    pub sidebar: Sidebar,

    pub exit: bool,
    pub fatal_error: Option<String>,

    pub shared_state: SharedState,

    pub input_thread: Option<std::thread::JoinHandle<()>>,
    pub eth_price_thread: Option<tokio::task::JoinHandle<()>>,
    pub assets_thread: Option<tokio::task::JoinHandle<()>>,
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load();

        Self {
            context: vec![Page::MainMenu(MainMenuPage::default())],
            sidebar: Sidebar::default(),

            exit: false,
            fatal_error: None,

            shared_state: SharedState {
                assets: None,
                current_account: config.current_account,
                online: None,
                eth_price: None,
                testnet_mode: config.testnet_mode,
                focus: Focus::Main,
            },

            input_thread: None,
            eth_price_thread: None,
            assets_thread: None,
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

    pub fn init_threads(&mut self, tr: &mpsc::Sender<Event>, sd: &Arc<AtomicBool>) {
        let tr_input = tr.clone();
        let shutdown_signal = sd.clone();
        self.input_thread = Some(std::thread::spawn(move || {
            events::input::watch_input_events(tr_input, shutdown_signal);
        }));

        let tr_eth_price = tr.clone();
        let shutdown_signal = sd.clone();
        self.eth_price_thread = Some(tokio::spawn(async move {
            events::eth_price::watch_eth_price_change(tr_eth_price, shutdown_signal).await
        }));
    }

    fn start_assets_thread(&mut self, tr: &mpsc::Sender<Event>, sd: &Arc<AtomicBool>) {
        if self.assets_thread.is_none() {
            let tr_assets = tr.clone();
            let shutdown_signal = sd.clone();
            self.assets_thread = Some(tokio::spawn(async move {
                events::assets::watch_assets(tr_assets, shutdown_signal).await
            }));
        }
    }

    async fn stop_assets_thread(&mut self) {
        if let Some(thread) = self.assets_thread.take() {
            thread.abort();
            let _ = thread.await;
        }
    }

    fn set_online(&mut self, tr: &mpsc::Sender<Event>, sd: &Arc<AtomicBool>) {
        self.start_assets_thread(tr, sd);

        self.shared_state.online = Some(true);
    }

    async fn set_offline(&mut self) {
        self.shared_state.online = Some(false);

        self.stop_assets_thread().await;
    }

    pub async fn exit_threads(&mut self) {
        if let Some(thread) = self.input_thread.take() {
            thread.join().unwrap();
        }
        if let Some(thread) = self.eth_price_thread.take() {
            thread.await.unwrap();
        }
        if let Some(thread) = self.assets_thread.take() {
            thread.await.unwrap();
        }

        for page in &mut self.context {
            page.exit_threads().await;
        }
    }

    pub fn reload(&mut self) {
        let config = Config::load();
        self.shared_state.testnet_mode = config.testnet_mode;
    }

    async fn process_result(
        &mut self,
        result: crate::Result<HandleResult>,
    ) -> crate::Result<usize> {
        let result = match result {
            Ok(res) => res,
            Err(error) => {
                self.fatal_error = Some(format!("{error:?}"));
                return Err(error);
            }
        };
        for _ in 0..result.page_pops {
            self.context.pop();
        }
        if result.reload {
            self.reload();
            if let Some(page) = self.current_page_mut() {
                page.reload();
            }
        }
        if result.refresh_assets {
            self.shared_state.assets = None;
            // TODO restart the assets thread to avoid the delay
        }
        if !result.page_inserts.is_empty() {
            // In case we are in the sidebar, we should switch to left side.
            self.shared_state.focus = Focus::Main;
        }
        self.context.extend(result.page_inserts);
        Ok(result.esc_ignores)
    }

    pub async fn handle_event(
        &mut self,
        event: super::events::Event,
        tr: &mpsc::Sender<Event>,
        sd: &Arc<AtomicBool>,
    ) -> crate::Result<()> {
        let mut esc_ignores = if self.fatal_error.is_none()
            && self.shared_state.focus == Focus::Main
            && let Some(page) = self.context.last_mut()
        {
            let result = page.handle_event(&event, tr, sd, &self.shared_state);
            self.process_result(result).await?
        } else {
            0
        };

        esc_ignores += if self.fatal_error.is_none() && self.shared_state.focus == Focus::Sidebar {
            let result = self
                .sidebar
                .handle_event(&event, tr, sd, &self.shared_state);
            self.process_result(result).await?
        } else {
            0
        };

        if self.context.is_empty() {
            self.exit = true;
        }

        match event {
            Event::Input(key_event) => {
                // check if we should exit on 'q' press
                if key_event.kind == KeyEventKind::Press {
                    #[allow(clippy::single_match)]
                    match key_event.code {
                        KeyCode::Left => {
                            if self.shared_state.focus == Focus::Sidebar {
                                self.shared_state.focus = Focus::Main;
                            }
                        }
                        KeyCode::Right => {
                            if self.shared_state.focus == Focus::Main
                                && !matches!(
                                    self.context.last(),
                                    Some(Page::AccountCreate(_)) | Some(Page::Trade(_))
                                )
                            {
                                self.shared_state.focus = Focus::Sidebar;
                            }
                        }
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
                            }
                            if char == 't' && key_event.modifiers == KeyModifiers::CONTROL {
                                self.context.push(Page::Trade(TradePage::default()));
                            }
                        }
                        KeyCode::Esc => {
                            if self.fatal_error.is_some() {
                                self.fatal_error = None;
                            } else if self.shared_state.focus == Focus::Sidebar {
                                self.shared_state.focus = Focus::Main;
                            } else if esc_ignores == 0 {
                                let page = self.context.pop();
                                if let Some(mut page) = page {
                                    page.exit_threads().await;
                                }
                                if self.context.is_empty() {
                                    self.exit = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            Event::AccountChange(address) => {
                self.shared_state.current_account = Some(address);
            }

            // ETH Price API
            Event::EthPriceUpdate(eth_price) => {
                self.shared_state.eth_price = Some(eth_price);
                self.set_online(tr, sd);
            }
            Event::EthPriceError(error) => {
                if error.is_connect() {
                    // ETH Price is the main API for understanding if we are connected to internet
                    self.set_offline().await;
                } else {
                    self.fatal_error = Some(error.fmt_err("EthPriceError"))
                }
            }

            // Assets API
            Event::AssetsUpdate(assets) => self.shared_state.assets = Some(assets),
            Event::AssetsUpdateError(error) => self.fatal_error = Some(error),

            // Candles API
            Event::CandlesUpdateError(error) => {
                self.fatal_error = Some(error.fmt_err("CandlesUpdateError"));
            }

            // Transaction API
            Event::TxSubmitError(error) => self.fatal_error = Some(error),
            Event::TxStatusError(error) => self.fatal_error = Some(error),

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
                current_account: self.shared_state.current_account.as_ref(),
                online: self.shared_state.online,
            }
            .render(title_area, buf);

            // Body render
            if page.is_full_screen() {
                page.render_component_with_block(
                    body_area,
                    buf,
                    Block::bordered(),
                    &self.shared_state,
                );
            } else {
                let horizontal_layout =
                    Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]);
                let [left_area, right_area] = horizontal_layout.areas(body_area);

                page.render_component_with_block(
                    left_area,
                    buf,
                    Block::bordered(),
                    &self.shared_state,
                );

                self.sidebar.render_component_with_block(
                    right_area,
                    buf,
                    Block::bordered().border_type(BorderType::Plain),
                    &self.shared_state,
                );
            }

            Footer {
                exit: &self.exit,
                is_main_menu: &page.is_main_menu(),
            }
            .render(footer_area, buf);
        }

        if let Some(fatal_error) = &self.fatal_error {
            Popup {
                bg_color: Some(Color::Red),
            }
            .render(area, buf);

            let popup_inner_area = Popup::inner_area(area);

            let block = Block::bordered().title("Fatal Error");
            Paragraph::new(Text::raw(fatal_error))
                .wrap(Wrap { trim: false })
                .to_owned()
                .render_with_block(popup_inner_area, buf, block);
        };
    }
}
