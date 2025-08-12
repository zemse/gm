use std::{
    io,
    str::FromStr,
    sync::{atomic::AtomicBool, mpsc, Arc, RwLock, RwLockWriteGuard},
};

use alloy::primitives::Address;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use pages::{
    config::ConfigPage,
    dev_key_capture::DevKeyCapturePage,
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
use widgets::{footer::Footer, form::Form, text_popup::TextPopup, title::Title};

use super::{
    events::{self, Event},
    traits::{Component, HandleResult, RectUtil},
};
use crate::{
    disk::{Config, DiskInterface},
    error::FmtError,
    tui::{app::widgets::invite_popup::InvitePopup, events::helios::helios_thread},
    utils::assets::Asset,
};
use crate::{
    tui::theme::{Theme, ThemeName},
    utils::assets::AssetManager,
};

pub mod pages;
pub mod widgets;

// TODO update focus to have title bar, footer
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Focus {
    Main,
    Sidebar,
}

pub struct SharedState {
    pub online: Option<bool>,
    pub asset_manager: Arc<RwLock<AssetManager>>,
    pub recent_addresses: Option<Vec<Address>>,
    pub testnet_mode: bool,
    pub developer_mode: bool,
    current_account: Option<Address>,
    pub alchemy_api_key_available: bool,
    pub eth_price: Option<String>,
    pub theme: Theme,
}

impl SharedState {
    pub fn assets_read(&self) -> crate::Result<Option<Vec<Asset>>> {
        let Some(current_account) = self.current_account else {
            return Ok(None);
        };

        Ok(self
            .asset_manager
            .read()
            .map_err(|e| format!("poison error - please restart gm - {e}"))?
            .get_assets(&current_account)
            .cloned())
    }

    pub fn assets_mut(&mut self) -> crate::Result<RwLockWriteGuard<'_, AssetManager>> {
        Ok(self
            .asset_manager
            .write()
            .map_err(|e| format!("poison error - please restart gm - {e}"))?)
    }

    pub fn try_current_account(&self) -> crate::Result<Address> {
        self.current_account
            .ok_or_else(|| crate::Error::CurrentAccountNotSet)
    }
}

pub struct App {
    pub context: Vec<Page>,
    pub preview_page: Option<Page>,

    pub exit: bool,
    pub fatal_error_popup: TextPopup,

    pub shared_state: SharedState,

    pub invite_popup: InvitePopup,

    pub input_thread: Option<std::thread::JoinHandle<()>>,
    pub eth_price_thread: Option<tokio::task::JoinHandle<()>>,
    pub assets_thread: Option<tokio::task::JoinHandle<()>>,
    pub recent_addresses_thread: Option<tokio::task::JoinHandle<()>>,
    pub helios_thread: Option<tokio::task::JoinHandle<()>>,
}

impl App {
    pub fn new() -> crate::Result<Self> {
        let config = Config::load()?;
        let theme_name = ThemeName::from_str(&config.theme_name)?;
        let theme = Theme::new(theme_name);
        Ok(Self {
            context: vec![Page::MainMenu(MainMenuPage::new(config.developer_mode)?)],
            preview_page: None,

            exit: false,
            // fatal_error: None,
            fatal_error_popup: TextPopup::new("Fatal Error"),
            shared_state: SharedState {
                asset_manager: Arc::new(RwLock::new(AssetManager::default())),
                recent_addresses: None,
                current_account: config.current_account,
                developer_mode: config.developer_mode,
                alchemy_api_key_available: config.alchemy_api_key.is_some(),
                online: None,
                eth_price: None,
                testnet_mode: config.testnet_mode,
                theme,
            },

            invite_popup: InvitePopup::default(),

            input_thread: None,
            eth_price_thread: None,
            assets_thread: None,
            recent_addresses_thread: None,
            helios_thread: None,
        })
    }

    pub fn cli_args(&mut self, args: Vec<String>) -> crate::Result<()> {
        if args.len() == 1 {
            // TODO support for wallet connect URI
            if args[0].ends_with("invite") {
                self.invite_popup.set_invite_code(&args[0]);
                self.invite_popup.open();
            }
        } else if args.len() > 1 {
            self.fatal_error_popup.set_text(format!(
                "Too many arguments provided to gm on the cli: {args:?}. Expected 0 or 1.",
            ));
        }

        Ok(())
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

        // TODO enable disabling helios through config
        if self.helios_thread.is_none() {
            let tr = tr.clone();
            let assets_manager = Arc::clone(&self.shared_state.asset_manager);
            self.helios_thread = Some(tokio::spawn(async move {
                if let Err(e) = helios_thread(&tr, assets_manager).await {
                    let _ = tr.send(Event::HeliosError(e.fmt_err("HeliosError")));
                }
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

    pub fn reload(&mut self) -> crate::Result<()> {
        let config = Config::load()?;
        self.shared_state.testnet_mode = config.testnet_mode;
        self.shared_state.alchemy_api_key_available = config.alchemy_api_key.is_some();
        self.shared_state.current_account = config.current_account;
        self.shared_state.developer_mode = config.developer_mode;
        let theme_name = ThemeName::from_str(&config.theme_name)?;
        let theme = Theme::new(theme_name);
        self.shared_state.theme = theme;
        for page in &mut self.context {
            page.reload(&self.shared_state)?;
        }

        Ok(())
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
            self.reload()?;
            if let Some(page) = self.context.last_mut() {
                page.reload(&self.shared_state)?;
            }
        }
        if result.refresh_assets {
            // TODO restart the assets thread to avoid the delay
            if let Some(account) = Config::current_account()? {
                self.shared_state.assets_mut()?.clear_data_for(account);
            }
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

        let result = if self.fatal_error_popup.is_shown() {
            self.fatal_error_popup.handle_event(&event, area)
        } else if self.invite_popup.is_open() {
            self.invite_popup.handle_event(&event, tr)
        } else if self.context.last().is_some() {
            let page = self.context.last_mut().unwrap();
            page.handle_event(&event, body_area.block_inner(), tr, sd, &self.shared_state)
        } else {
            Ok(HandleResult::default())
        };

        let esc_ignores = self.process_result(result).await?;

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
                self.reload()?;
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
            Event::AssetsUpdate(wallet_address, assets) => {
                self.shared_state
                    .assets_mut()?
                    .update_assets(wallet_address, assets)?;
            }
            Event::AssetsUpdateError(error, silence_error) => {
                if !silence_error {
                    self.fatal_error_popup.set_text(error);
                }
            }

            Event::HeliosUpdate {
                account,
                network,
                token_address,
                status,
            } => {
                self.shared_state
                    .asset_manager
                    .write()
                    .map_err(|err| format!("poison error - please restart gm - {err}"))?
                    .update_light_client_verification(account, network, token_address, status);
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
            Event::TxError(error) => self.fatal_error_popup.set_text(error),

            Event::WalletConnectError(_, error) => {
                self.fatal_error_popup.set_text(error);
            }

            _ => {}
        };

        Ok(())
    }

    fn current_page(&self) -> Option<&Page> {
        self.context.last()
    }

    // TODO using this triggers rust borrow checks, as we are not able to do
    // immutable borrows once self is borrowed mutably
    #[allow(dead_code)]
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

        Title.render_component(title_area, buf, &self.shared_state);

        if let Some(page) = self.current_page() {
            // Render Body
            if page.is_main_menu() && page.main_menu_focused_item().is_some() {
                let main_menu_item = page.main_menu_focused_item().unwrap();
                let [left_area, gap_area, right_area] = Layout::horizontal([
                    Constraint::Length(13),
                    Constraint::Length(1),
                    Constraint::Min(2),
                ])
                .areas(body_area.block_inner());

                // Render border of the canvas
                Block::bordered()
                    .border_type(self.shared_state.theme.border_type)
                    .render(body_area, buf);

                // Render the Middle stick
                let height_inner = gap_area.height;
                let mut gap_area = gap_area.expand_vertical(1);
                "┬".render(gap_area, buf);
                for _ in 0..height_inner {
                    gap_area = gap_area.consume_height(1).expect("should not fail");
                    "│".render(gap_area, buf);
                }
                gap_area = gap_area.consume_height(1).expect("should not fail");
                "┴".render(gap_area, buf);

                // Render Main Menu on the Left side
                page.render_component(left_area, buf, &self.shared_state);

                // Render the preview of selection on the Right side
                let dummy_page = match main_menu_item {
                    MainMenuItem::Portfolio => {
                        let mut preview_page = main_menu_item
                            .get_page()
                            .expect("main_menu_item.get_page() failed");
                        preview_page.set_focus(false);
                        preview_page
                    }
                    MainMenuItem::Config => Page::Config(ConfigPage {
                        form: Form::init(|form| {
                            form.show_everything_empty(true);
                            Ok(())
                        })
                        .unwrap(),
                    }),
                    MainMenuItem::CompleteSetup => Page::Text(TextPage::new(
                        "Setup some of the essential stuff to get the most out of gm".to_string(),
                    )),
                    MainMenuItem::Accounts => {
                        Page::Text(TextPage::new("Create or load accounts".to_string()))
                    }
                    MainMenuItem::AddressBook => {
                        Page::Text(TextPage::new("Manage familiar addresses".to_string()))
                    }
                    MainMenuItem::Networks => {
                        Page::Text(TextPage::new("Manage networks and tokens".to_string()))
                    }
                    MainMenuItem::WalletConnect => {
                        let mut preview_page = main_menu_item
                            .get_page()
                            .expect("main_menu_item.get_page() failed");
                        preview_page.set_focus(false);
                        preview_page
                    }
                    MainMenuItem::SignMessage => Page::Text(TextPage::new(
                        "Sign a message and prove ownership to somebody".to_string(),
                    )),
                    MainMenuItem::SendMessage => {
                        Page::Text(TextPage::new("Send onchain message to someone".to_string()))
                    }
                    MainMenuItem::DevKeyInput => Page::DevKeyCapture(DevKeyCapturePage::default()),
                };

                dummy_page.render_component(right_area, buf, &self.shared_state);
            } else {
                page.render_component_with_block(
                    body_area,
                    buf,
                    Block::bordered().border_type(self.shared_state.theme.border_type),
                    &self.shared_state,
                );
            }

            Footer {
                exit: &self.exit,
                is_main_menu: &page.is_main_menu(),
            }
            .render(footer_area, buf, &self.shared_state.theme);
        }

        self.invite_popup.render(area, buf, &self.shared_state);

        self.fatal_error_popup
            .render(area, buf, &self.shared_state.theme.error_popup());
    }
}
