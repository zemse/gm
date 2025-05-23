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
    traits::{BorderedWidget, Component},
};

pub mod pages;
pub mod widgets;

pub struct SharedState<'a> {
    pub assets: &'a Option<Vec<Asset>>,
}

pub struct App {
    pub exit: bool,
    pub current_account: Option<Address>,
    pub context: Vec<Page>,
    pub online: Option<bool>,
    pub eth_price: Option<String>,
    pub assets: Option<Vec<Asset>>,
    pub testnet_mode: bool,
    pub fatal_error: Option<String>,

    pub input_thread: Option<std::thread::JoinHandle<()>>,
    pub eth_price_thread: Option<tokio::task::JoinHandle<()>>,
    pub assets_thread: Option<tokio::task::JoinHandle<()>>,
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load();

        Self {
            exit: false,
            current_account: config.current_account,
            context: vec![Page::MainMenu(MainMenuPage::default())],
            online: None,
            eth_price: None,
            assets: None,
            testnet_mode: config.testnet_mode,
            fatal_error: None,
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

        // self.set_online(tr, sd);
    }

    fn set_online(&mut self, tr: &mpsc::Sender<Event>, sd: &Arc<AtomicBool>) {
        if self.assets_thread.is_none() {
            let tr_assets = tr.clone();
            let shutdown_signal = sd.clone();
            self.assets_thread = Some(tokio::spawn(async move {
                events::assets::watch_assets(tr_assets, shutdown_signal).await
            }));
        }

        self.online = Some(true);
    }

    async fn set_offline(&mut self) {
        self.online = Some(false);

        if let Some(thread) = self.assets_thread.take() {
            thread.abort();
            let _ = thread.await;
        }
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
        self.testnet_mode = config.testnet_mode;
    }

    pub async fn handle_event(
        &mut self,
        event: super::events::Event,
        tr: &mpsc::Sender<Event>,
        sd: &Arc<AtomicBool>,
    ) -> crate::Result<()> {
        let esc_ignores = if self.fatal_error.is_none()
            && let Some(page) = self.current_page_mut()
        {
            let result = match page.handle_event(&event, tr, sd) {
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
            self.context.extend(result.page_inserts);
            result.esc_ignores
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
                            if char == 't' && key_event.modifiers == KeyModifiers::CONTROL {
                                self.context.push(Page::Trade(TradePage::default()));
                            }
                        }
                        KeyCode::Esc => {
                            if self.fatal_error.is_some() {
                                self.fatal_error = None;
                                // self.shared_state.cursor_freeze = false;
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
                self.current_account = Some(address);
            }

            // ETH Price API
            Event::EthPriceUpdate(eth_price) => {
                self.eth_price = Some(eth_price);
                self.set_online(tr, sd);
            }
            Event::EthPriceError(error) => {
                if error.is_connect() {
                    // ETH Price is the main API for understanding if we are connected to internet
                    self.set_offline().await;
                } else {
                    self.fatal_error = Some(error.fmt_err())
                }
            }

            // Assets API
            Event::AssetsUpdate(assets) => self.assets = Some(assets),
            Event::AssetsUpdateError(error) => self.fatal_error = Some(error),

            // Candles API
            Event::CandlesUpdateError(error) => {
                if error.is_connect() {
                    self.fatal_error = Some(format!("Please ensure internet access\n{error:?}"))
                } else {
                    self.fatal_error = Some(error.to_string())
                }
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
                online: self.online,
            }
            .render(title_area, buf);

            let app_shared_state = SharedState {
                assets: &self.assets,
            };

            // Body render
            if page.is_full_screen() {
                page.render_component_with_block(
                    body_area,
                    buf,
                    Block::bordered(),
                    &app_shared_state,
                );
            } else {
                let horizontal_layout =
                    Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]);
                let [left_area, right_area] = horizontal_layout.areas(body_area);

                page.render_component_with_block(
                    left_area,
                    buf,
                    Block::bordered(),
                    &app_shared_state,
                );

                Sidebar {
                    online: &self.online,
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
