#[cfg(feature = "demo")]
use std::time::Instant;
use std::{
    io::{self, stdout},
    str::FromStr,
    sync::{mpsc, Arc, RwLock, RwLockWriteGuard},
};

#[cfg(feature = "demo")]
use crate::demo::{demo_exit_text, demo_text, demo_text_2};
use crate::{
    events::{
        assets::watch_assets, input::watch_input_events, recent_addresses::watch_recent_addresses,
    },
    pages::{
        config::ConfigPage,
        dev_key_capture::DevKeyCapturePage,
        footer::Footer,
        invite_popup::InvitePopup,
        main_menu::{MainMenuItem, MainMenuPage},
        shell::ShellPage,
        text::TextPage,
        title::Title,
        trade::TradePage,
        Page,
    },
    AppEvent,
};
use alloy::primitives::Address;
use gm_ratatui_extra::{
    extensions::RectExt, form::Form, text_popup::TextPopup, thematize::Thematize,
};
use gm_utils::{
    assets::{Asset, AssetManager},
    config::Config,
    disk_storage::DiskStorageInterface,
    network::NetworkStore,
    price_manager::PriceManager,
};
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
    DefaultTerminal,
};
use tokio_util::sync::CancellationToken;

use super::traits::{Actions, Component};
use crate::{
    error::FmtError,
    events::helios::helios_thread,
    theme::{Theme, ThemeName},
};

pub struct SharedState {
    pub online: Option<bool>,
    pub asset_manager: Arc<RwLock<AssetManager>>,
    pub price_manager: Arc<PriceManager>,
    pub recent_addresses: Option<Vec<Address>>,
    pub testnet_mode: bool,
    pub developer_mode: bool,
    pub current_account: Option<Address>,
    pub alchemy_api_key_available: bool,
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
            .map_err(|_| crate::Error::Poisoned("assets_read".to_string()))?
            .get_assets(&current_account)
            .cloned())
    }

    pub fn assets_mut(&mut self) -> crate::Result<RwLockWriteGuard<'_, AssetManager>> {
        self.asset_manager
            .write()
            .map_err(|_| crate::Error::Poisoned("assets_mut".to_string()))
    }

    pub fn try_current_account(&self) -> crate::Result<Address> {
        self.current_account
            .ok_or_else(|| crate::Error::CurrentAccountNotSet)
    }
}

pub struct App {
    exit: bool,
    context: Vec<Page>,
    shared_state: SharedState,

    fatal_error_popup: TextPopup,
    pub invite_popup: InvitePopup,
    #[cfg(feature = "demo")]
    demo_popup: TextPopup,

    input_thread: Option<std::thread::JoinHandle<()>>,
    refresh_prices_thread: Option<tokio::task::JoinHandle<()>>,
    assets_thread: Option<tokio::task::JoinHandle<()>>,
    recent_addresses_thread: Option<tokio::task::JoinHandle<()>>,
    helios_thread: Option<tokio::task::JoinHandle<()>>,

    #[cfg(feature = "demo")]
    demo_timer: Option<Instant>,
}

impl App {
    pub fn new() -> crate::Result<Self> {
        let networks = NetworkStore::load_and_update()?;

        let config = Config::load()?;
        let theme_name = ThemeName::from_str(config.get_theme_name())?;
        let theme = Theme::new(theme_name);
        Ok(Self {
            exit: false,
            context: vec![Page::MainMenu(MainMenuPage::new(
                config.get_developer_mode(),
            )?)],
            shared_state: SharedState {
                asset_manager: Arc::new(RwLock::new(AssetManager::default())),
                price_manager: Arc::new(PriceManager::new(networks.networks)?),
                recent_addresses: None,
                current_account: config.get_current_account().ok(),
                developer_mode: config.get_developer_mode(),
                alchemy_api_key_available: config.get_developer_mode(),
                online: None,
                testnet_mode: config.get_testnet_mode(),
                theme,
            },

            fatal_error_popup: TextPopup::new("Fatal Error", true),
            invite_popup: InvitePopup::default(),
            #[cfg(feature = "demo")]
            demo_popup: TextPopup::new("", false),

            input_thread: None,
            refresh_prices_thread: None,
            assets_thread: None,
            recent_addresses_thread: None,
            helios_thread: None,

            #[cfg(feature = "demo")]
            demo_timer: Some(Instant::now()),
        })
    }

    pub async fn run(&mut self, pre_events: Option<Vec<AppEvent>>) -> crate::Result<()> {
        let (event_tr, event_rc) = mpsc::channel::<AppEvent>();
        let shutdown = CancellationToken::new();
        let mut terminal = ratatui::init();
        execute!(stdout(), EnableMouseCapture)?;

        self.init_threads(&event_tr, &shutdown);

        #[cfg(feature = "demo")]
        self.demo_popup.set_text(demo_text().to_string());

        if let Some(events) = pre_events {
            let area = self.draw(&mut terminal).map_err(crate::Error::Draw)?;
            for event in events {
                self.handle_event(event, area, &event_tr, &shutdown)
                    .await
                    .unwrap_or_else(|e| {
                        self.fatal_error_popup.set_text(e.to_string());
                    })
            }
        }

        while !self.exit {
            let area = self.draw(&mut terminal).map_err(crate::Error::Draw)?;

            self.handle_event(event_rc.recv()?, area, &event_tr, &shutdown)
                .await
                .unwrap_or_else(|e| self.fatal_error_popup.set_text(e.to_string()));
        }

        // final render before exiting
        self.draw(&mut terminal).map_err(crate::Error::Draw)?;

        // signal all the threads to exit
        shutdown.cancel();
        self.exit_threads().await;

        ratatui::restore();
        execute!(stdout(), DisableMouseCapture)?;

        #[cfg(feature = "demo")]
        println!("{}", demo_exit_text());

        Ok(())
    }

    fn draw(&self, terminal: &mut DefaultTerminal) -> io::Result<Rect> {
        let completed_frame = terminal.draw(|frame| {
            frame.render_widget(self, frame.area());
        })?;
        Ok(completed_frame.area)
    }

    fn init_threads(&mut self, tr: &mpsc::Sender<AppEvent>, sd: &CancellationToken) {
        let tr_input = tr.clone();
        let shutdown_signal = sd.clone();
        self.input_thread = Some(std::thread::spawn(move || {
            watch_input_events(tr_input, shutdown_signal);
        }));

        let tr_eth_price = tr.clone();
        let shutdown_signal = sd.clone();
        self.refresh_prices_thread =
            Some(self.shared_state.price_manager.spawn_refresh_prices_thread(
                shutdown_signal,
                move |res| {
                    let _ = match res {
                        Ok(()) => tr_eth_price.send(AppEvent::PricesUpdate),
                        Err(e) => tr_eth_price.send(AppEvent::PricesUpdateError(e)),
                    };
                },
            ));
    }

    fn start_other_threads(
        &mut self,
        tr: &mpsc::Sender<AppEvent>,
        sd: &CancellationToken,
    ) -> crate::Result<()> {
        if self.assets_thread.is_none() {
            let tr_assets = tr.clone();
            let shutdown_signal = sd.clone();
            self.assets_thread = Some(tokio::spawn(async move {
                watch_assets(tr_assets, shutdown_signal).await
            }));
        };

        if self.helios_thread.is_none() && Config::load()?.get_helios_enabled() {
            let tr = tr.clone();
            let sd = sd.clone();
            let assets_manager = Arc::clone(&self.shared_state.asset_manager);
            self.helios_thread = Some(tokio::spawn(async move {
                if let Err(e) = helios_thread(&tr, &sd, assets_manager).await {
                    let _ = tr.send(AppEvent::HeliosError(e.fmt_err("HeliosError")));
                }
            }));
        }

        if self.recent_addresses_thread.is_none() {
            let tr_recent_addresses = tr.clone();
            let shutdown_signal = sd.clone();
            self.recent_addresses_thread = Some(tokio::spawn(async move {
                watch_recent_addresses(tr_recent_addresses, shutdown_signal).await
            }));
        }

        Ok(())
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

    fn set_online(
        &mut self,
        tr: &mpsc::Sender<AppEvent>,
        sd: &CancellationToken,
    ) -> crate::Result<()> {
        self.shared_state.online = Some(true);

        self.start_other_threads(tr, sd)
    }

    async fn set_offline(&mut self) {
        self.shared_state.online = Some(false);

        self.stop_other_threads().await;
    }

    pub async fn exit_threads(&mut self) {
        if let Some(thread) = self.input_thread.take() {
            thread.join().unwrap();
        }

        if let Some(thread) = self.refresh_prices_thread.take() {
            thread.await.unwrap();
        }

        if let Some(thread) = self.assets_thread.take() {
            thread.await.unwrap();
        }

        if let Some(thread) = self.recent_addresses_thread.take() {
            thread.await.unwrap();
        }

        if let Some(thread) = self.helios_thread.take() {
            thread.await.unwrap();
        }

        for page in &mut self.context {
            page.exit_threads().await;
        }
    }

    fn reload(&mut self) -> crate::Result<()> {
        let config = Config::load()?;
        self.shared_state.testnet_mode = config.get_testnet_mode();
        self.shared_state.alchemy_api_key_available = config.get_alchemy_api_key(false).is_ok();
        self.shared_state.current_account = config.get_current_account().ok();
        self.shared_state.developer_mode = config.get_developer_mode();
        let theme_name = ThemeName::from_str(config.get_theme_name())?;
        let theme = Theme::new(theme_name);
        self.shared_state.theme = theme;
        for page in &mut self.context {
            page.reload(&self.shared_state)?;
        }

        Ok(())
    }

    async fn process_result(&mut self, result: Actions) -> crate::Result<(bool, bool)> {
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
            if let Ok(account) = Config::current_account() {
                self.shared_state.assets_mut()?.clear_data_for(account);
            }
        }
        self.context.extend(result.page_inserts);
        Ok((result.ignore_esc, result.ignore_ctrlc))
    }

    async fn handle_event(
        &mut self,
        event: AppEvent,
        area: Rect,
        tr: &mpsc::Sender<AppEvent>,
        sd: &CancellationToken,
    ) -> crate::Result<()> {
        let [_, body_area, _] = self.get_areas(area);

        #[cfg(feature = "demo")]
        if let Some(demo_timer) = self.demo_timer {
            use crate::demo::DEMO_2_DELAY;

            if demo_timer.elapsed() >= DEMO_2_DELAY {
                self.demo_timer = None;
                self.demo_popup.set_text(demo_text_2().to_string());
            }
        }

        #[cfg(feature = "demo")]
        let demo_popup_shown = self.demo_popup.is_shown();
        #[cfg(not(feature = "demo"))]
        let demo_popup_shown = false;

        // Update state based on events
        match &event {
            AppEvent::AccountChange(address) => {
                self.shared_state.current_account = Some(*address);
            }

            AppEvent::ConfigUpdate => {
                self.reload()?;
            }

            AppEvent::PricesUpdate => {
                // The prices have already been updated in the PriceManager store in shared_state
                self.set_online(tr, sd)?;
            }
            AppEvent::PricesUpdateError(error) => {
                if error.is_connect() {
                    // ETH Price is the main API for understanding if we are connected to internet
                    self.set_offline().await;
                } else {
                    self.fatal_error_popup.set_text(error.to_string());
                }
            }

            // Assets API
            AppEvent::AssetsUpdate(wallet_address, assets) => {
                self.shared_state
                    .assets_mut()?
                    // TODO use Option RefCell here to avoid clone
                    .update_assets(*wallet_address, assets.clone())?;
            }
            AppEvent::AssetsUpdateError(error, silence_error) => {
                if !silence_error {
                    self.fatal_error_popup.set_text(error.to_string());
                }
            }

            AppEvent::HeliosUpdate {
                account,
                network,
                token_address,
                status,
            } => {
                self.shared_state
                    .asset_manager
                    .write()
                    .map_err(|_| crate::Error::Poisoned("HeliosUpdate".to_string()))?
                    .update_light_client_verification(
                        *account,
                        // TODO use Option RefCell here to avoid clone
                        network.clone(),
                        token_address.clone(),
                        status.clone(),
                    );
            }
            AppEvent::HeliosError(error) => {
                self.fatal_error_popup.set_text(error.clone());
            }

            AppEvent::RecentAddressesUpdate(addresses) => {
                self.shared_state.recent_addresses = Some(addresses.clone());
            }
            AppEvent::RecentAddressesUpdateError(error) => {
                self.fatal_error_popup.set_text(error.to_string());
            }

            // Candles API
            AppEvent::CandlesUpdateError(error) => {
                self.fatal_error_popup.set_text(error.to_string());
            }

            // Transaction API
            AppEvent::TxError(error) => self.fatal_error_popup.set_text(error.clone()),

            AppEvent::WalletConnectError(_, error) => {
                self.fatal_error_popup.set_text(error.clone());
            }

            AppEvent::InviteError(error) => {
                self.fatal_error_popup.set_text(error.clone());
            }

            _ => {}
        }

        // Handle the event in the relavent component
        let result = if self.fatal_error_popup.is_shown() {
            self.fatal_error_popup
                .handle_event::<Actions>(event.key_event(), area)
        } else if self.invite_popup.is_open() {
            self.invite_popup
                .handle_event(&event, tr, &self.shared_state)?
        } else if demo_popup_shown {
            #[cfg(not(feature = "demo"))]
            unreachable!();
            #[cfg(feature = "demo")]
            self.demo_popup
                .handle_event::<Actions>(event.key_event(), area)
        } else if self.context.last().is_some() {
            let page = self.context.last_mut().unwrap();
            let page_area = if page.is_main_menu() {
                let [left_area, _, _] = Layout::horizontal([
                    Constraint::Length(13),
                    Constraint::Length(1),
                    Constraint::Min(2),
                ])
                .areas(body_area.block_inner());
                left_area
            } else {
                body_area.block_inner()
            };
            page.handle_event(&event, page_area, tr, sd, &self.shared_state)?
        } else {
            Actions::default()
        };

        let (ignore_esc, ignore_ctrlc) = self.process_result(result).await?;

        // Context is empty (due to pressing ESC)
        if self.context.is_empty() {
            self.exit = true;
        }

        // Global key handling
        if let AppEvent::Input(Event::Key(key_event)) = event {
            // check if we should exit on 'q' press
            if key_event.kind == KeyEventKind::Press {
                #[allow(clippy::single_match)]
                match key_event.code {
                    KeyCode::Char(char) => {
                        // TODO can we quit using q as well?
                        // if char == 'q' && self.navigation.text_input().is_none() {
                        //     self.exit = true;
                        // }
                        if char == 'c'
                            && key_event.modifiers == KeyModifiers::CONTROL
                            && !ignore_ctrlc
                        {
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
                        } else if !ignore_esc {
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
        };

        Ok(())
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

    pub fn current_page(&self) -> Option<&Page> {
        self.context.last()
    }

    pub fn current_page_mut(&mut self) -> Option<&mut Page> {
        self.context.last_mut()
    }

    pub fn insert_page(&mut self, page: Page) {
        self.context.push(page);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if area.width < 30 || area.height < 20 {
            "Increase size of your terminal please".render(area, buf);
            return;
        }

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
                    gap_area.consume_height(1);
                    "│".render(gap_area, buf);
                }
                gap_area.consume_height(1);
                "┴".render(gap_area, buf);

                // Render Main Menu on the Left side
                page.render_component(left_area, buf, &self.shared_state);

                // Render the preview of selection on the Right side
                let dummy_page = match main_menu_item {
                    MainMenuItem::Portfolio => {
                        let mut preview_page = main_menu_item
                            .get_page(&self.shared_state)
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
                            .get_page(&self.shared_state)
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
                    MainMenuItem::Shell => Page::Shell(ShellPage::default()),
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

        #[cfg(feature = "demo")]
        self.demo_popup
            .render(area, buf, &self.shared_state.theme.popup());
    }
}
