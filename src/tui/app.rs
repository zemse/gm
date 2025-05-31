use std::{
    io,
    sync::{atomic::AtomicBool, mpsc, Arc},
};

use alloy::primitives::Address;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use pages::{
    config::ConfigPage,
    main_menu::{MainMenuItem, MainMenuPage},
    text::TextPage,
    trade::TradePage,
    Page,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
    DefaultTerminal,
};
use widgets::{
    footer::Footer, form::Form, popup::Popup, sidebar::Sidebar, text_popup::TextPopup, title::Title,
};

use crate::{
    disk::{Config, DiskInterface},
    error::FmtError,
    utils::assets::Asset,
};

use super::{
    events::{self, Event},
    traits::{Component, HandleResult},
};

pub mod pages;
pub mod widgets;

#[derive(Debug, PartialEq)]
pub enum Focus {
    Main,
    Sidebar,
}

pub struct SharedState {
    pub online: Option<bool>,
    pub assets: Option<Vec<Asset>>,
    pub recent_addresses: Option<Vec<Address>>,
    pub testnet_mode: bool,
    pub current_account: Option<Address>,
    pub alchemy_api_key_available: bool,
    pub eth_price: Option<String>,
    pub focus: Focus,
}

pub struct App {
    pub context: Vec<Page>,
    pub preview_page: Option<Page>,

    pub sidebar: Sidebar,

    pub exit: bool,
    // pub fatal_error: Option<String>,
    pub fatal_error_popup: TextPopup,

    pub shared_state: SharedState,

    pub input_thread: Option<std::thread::JoinHandle<()>>,
    pub eth_price_thread: Option<tokio::task::JoinHandle<()>>,
    pub assets_thread: Option<tokio::task::JoinHandle<()>>,
    pub recent_addresses_thread: Option<tokio::task::JoinHandle<()>>,
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load();

        Self {
            context: vec![Page::MainMenu(MainMenuPage::default())],
            preview_page: None,

            sidebar: Sidebar::default(),

            exit: false,
            // fatal_error: None,
            fatal_error_popup: TextPopup::new("Fatal Error"),

            shared_state: SharedState {
                assets: None,
                recent_addresses: None,
                current_account: config.current_account,
                alchemy_api_key_available: config.alchemy_api_key.is_some(),
                online: None,
                eth_price: None,
                testnet_mode: config.testnet_mode,
                focus: Focus::Main,
            },

            input_thread: None,
            eth_price_thread: None,
            assets_thread: None,
            recent_addresses_thread: None,
        }
    }
}

impl App {
    pub fn draw(&self, terminal: &mut DefaultTerminal) -> io::Result<Rect> {
        let completed_frame = terminal.draw(|frame| {
            frame.render_widget(self, frame.area());
        })?;
        Ok(completed_frame.area)
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

    fn start_other_threads(&mut self, tr: &mpsc::Sender<Event>, sd: &Arc<AtomicBool>) {
        if self.assets_thread.is_none() {
            let tr_assets = tr.clone();
            let shutdown_signal = sd.clone();
            self.assets_thread = Some(tokio::spawn(async move {
                events::assets::watch_assets(tr_assets, shutdown_signal).await
            }));
        }

        if self.recent_addresses_thread.is_none() {
            let tr_recent_addresses = tr.clone();
            let shutdown_signal = sd.clone();
            self.recent_addresses_thread = Some(tokio::spawn(async move {
                events::recent_addresses::watch_recent_addresses(
                    tr_recent_addresses,
                    shutdown_signal,
                )
                .await
            }));
        }
    }

    async fn stop_other_threads(&mut self) {
        if let Some(thread) = self.assets_thread.take() {
            thread.abort();
            let _ = thread.await;
        }

        if let Some(thread) = self.recent_addresses_thread.take() {
            thread.abort();
            let _ = thread.await;
        }
    }

    fn set_online(&mut self, tr: &mpsc::Sender<Event>, sd: &Arc<AtomicBool>) {
        self.start_other_threads(tr, sd);

        self.shared_state.online = Some(true);
    }

    async fn set_offline(&mut self) {
        self.shared_state.online = Some(false);

        self.stop_other_threads().await;
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
        self.shared_state.alchemy_api_key_available = config.alchemy_api_key.is_some();
        self.shared_state.current_account = config.current_account;

        for page in &mut self.context {
            page.reload();
        }
    }

    async fn process_result(
        &mut self,
        result: crate::Result<HandleResult>,
    ) -> crate::Result<usize> {
        let result = match result {
            Ok(res) => res,
            Err(error) => {
                self.fatal_error_popup.set_text(format!("{error:#?}"));
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
        area: Rect,
        tr: &mpsc::Sender<Event>,
        sd: &Arc<AtomicBool>,
    ) -> crate::Result<()> {
        let [_, body_area, _] = self.get_areas(area);

        // supply events to pages
        let mut esc_ignores = if (!event.is_input()
            || (self.shared_state.focus == Focus::Main && !self.fatal_error_popup.is_shown()))
            && let Some(page) = self.context.last_mut()
        {
            let result = page.handle_event(&event, body_area, tr, sd, &self.shared_state);
            self.process_result(result).await?
        } else {
            0
        };

        // suppy event to fatal error popup
        let result = self
            .fatal_error_popup
            .handle_event(&event, Popup::inner_area(area));
        esc_ignores += self.process_result(result).await?;

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
                                let _ = tr.send(Event::FocusChange(Focus::Main));
                            }
                        }
                        KeyCode::Right => {
                            if self.shared_state.focus == Focus::Main
                                && self
                                    .context
                                    .last()
                                    .map(|p| !p.is_full_screen())
                                    .unwrap_or(false)
                            {
                                self.shared_state.focus = Focus::Sidebar;
                                let _ = tr.send(Event::FocusChange(Focus::Sidebar));
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
                            if char == 'r' && key_event.modifiers == KeyModifiers::CONTROL {
                                self.fatal_error_popup.set_text("test error".to_string());
                            }
                            if char == 't' && key_event.modifiers == KeyModifiers::CONTROL {
                                self.context.push(Page::Trade(TradePage::default()));
                            }
                        }
                        KeyCode::Esc => {
                            if self.fatal_error_popup.is_shown() {
                                self.fatal_error_popup.clear();
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

            Event::ConfigUpdate => {
                self.reload();
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
                    self.fatal_error_popup
                        .set_text(error.fmt_err("EthPriceError"));
                }
            }

            // Assets API
            Event::AssetsUpdate(assets) => self.shared_state.assets = Some(assets),
            Event::AssetsUpdateError(error, silence_error) => {
                if !silence_error {
                    self.fatal_error_popup.set_text(error);
                }
            }

            Event::RecentAddressesUpdate(addresses) => {
                self.shared_state.recent_addresses = Some(addresses);
            }
            Event::RecentAddressesUpdateError(error) => {
                self.fatal_error_popup.set_text(error);
            }

            // Candles API
            Event::CandlesUpdateError(error) => {
                self.fatal_error_popup
                    .set_text(error.fmt_err("CandlesUpdateError"));
            }

            // Transaction API
            Event::TxSubmitError(error) => self.fatal_error_popup.set_text(error),
            Event::TxStatusError(error) => self.fatal_error_popup.set_text(error),

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

    fn get_areas(&self, area: Rect) -> [Rect; 3] {
        let [title_area, body_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .areas(area);
        [title_area, body_area, footer_area]
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [title_area, body_area, footer_area] = self.get_areas(area);

        Title {
            current_account: self.shared_state.current_account.as_ref(),
            online: self.shared_state.online,
        }
        .render(title_area, buf);

        if let Some(page) = self.current_page() {
            // Render Body
            if page.is_main_menu()
                && let Some(main_menu_item) = page.main_menu_focused_item()
            {
                let [left_area, right_area] =
                    Layout::horizontal([Constraint::Length(15), Constraint::Min(2)])
                        .areas(body_area);

                page.render_component_with_block(
                    left_area,
                    buf,
                    Block::bordered(),
                    &self.shared_state,
                );

                let page = match main_menu_item {
                    MainMenuItem::Assets => {
                        let mut preview_page = main_menu_item.get_page();
                        preview_page.set_focus(false);
                        preview_page
                    }
                    MainMenuItem::Config => Page::Config(ConfigPage {
                        form: Form::init(|form| {
                            form.show_everything_empty(true);
                        }),
                    }),
                    MainMenuItem::Setup => Page::Text(TextPage::new(
                        "Setup some of the essential stuff to get the most out of gm".to_string(),
                    )),
                    MainMenuItem::Accounts => {
                        Page::Text(TextPage::new("Create or load accounts".to_string()))
                    }
                    MainMenuItem::AddressBook => {
                        Page::Text(TextPage::new("Manage familiar addresses".to_string()))
                    }
                    MainMenuItem::SignMessage => Page::Text(TextPage::new(
                        "Sign a message and prove ownership to somebody".to_string(),
                    )),
                    MainMenuItem::SendMessage => {
                        Page::Text(TextPage::new("Send onchain message to someone".to_string()))
                    }
                };

                page.render_component_with_block(
                    right_area,
                    buf,
                    Block::bordered(),
                    &self.shared_state,
                );
            } else {
                page.render_component_with_block(
                    body_area,
                    buf,
                    Block::bordered(),
                    &self.shared_state,
                );
            }

            Footer {
                exit: &self.exit,
                is_main_menu: &page.is_main_menu(),
            }
            .render(footer_area, buf);
        }

        self.fatal_error_popup.render(area, buf);
    }
}
