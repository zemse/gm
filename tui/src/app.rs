#[cfg(feature = "demo")]
use std::time::Instant;
use std::{
    borrow::Cow,
    io::{self, stdout},
    ops::Mul,
    str::FromStr,
    sync::{mpsc, Arc, RwLock, RwLockWriteGuard},
    time::Duration,
};

#[cfg(feature = "demo")]
use crate::demo::{demo_exit_text, demo_text, demo_text_2};
use crate::{
    pages::{
        account::AccountPage, address_book::AddressBookPage, assets::AssetsPage,
        complete_setup::CompleteSetupPage, config::ConfigPage, dev_key_capture::DevKeyCapturePage,
        footer::Footer, invite_popup::InvitePopup, network::NetworkPage,
        send_message::SendMessagePage, shell::ShellPage, sign_message::SignMessagePage,
        title::Title, trade::TradePage, walletconnect::WalletConnectPage, Page,
    },
    post_handle_event::PostHandleEventActions,
    threads::{
        assets::watch_assets, helios::helios_thread, input::watch_input_events,
        recent_addresses::watch_recent_addresses, tick::start_ticking,
    },
    AppEvent,
};
use alloy::primitives::Address;
use arboard::Clipboard;
use gm_ratatui_extra::{
    act::Act,
    extensions::{RectExt, ThemedWidget},
    popup::PopupWidget,
    select::{Select, SelectEvent},
    text_popup::TextPopup,
    thematize::Thematize,
    toast::Toast,
};
use gm_utils::{
    assets::{Asset, AssetManager},
    config::Config,
    disk_storage::DiskStorageInterface,
    network::NetworkStore,
    price_manager::PriceManager,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Span,
    widgets::Widget,
    DefaultTerminal,
};
use ratatui::{
    crossterm::{
        event::{
            DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
        },
        execute,
    },
    widgets::Block,
};
use strum::{Display, EnumIter, IntoEnumIterator};
use tokio_util::sync::CancellationToken;

use super::traits::Component;
use crate::{
    error::FmtError,
    theme::{Theme, ThemeName},
};

#[derive(Debug, Display, EnumIter, PartialEq)]
pub enum MainMenuItem {
    CompleteSetup,
    Portfolio,
    Accounts,
    AddressBook,
    Networks,
    WalletConnect,
    SignMessage,
    SendMessage,
    DevKeyInput,
    Shell,
    Config,
}

impl MainMenuItem {
    pub fn get_page(&self, shared_state: &SharedState) -> crate::Result<Page> {
        Ok(match self {
            MainMenuItem::CompleteSetup => Page::CompleteSetup(CompleteSetupPage::new()?),
            MainMenuItem::Portfolio => Page::Assets(AssetsPage::new(shared_state.assets_read()?)?),
            MainMenuItem::Accounts => Page::Account(AccountPage::new()?),
            MainMenuItem::AddressBook => Page::AddressBook(AddressBookPage::new()?),
            MainMenuItem::Networks => Page::Network(NetworkPage::new()?),
            MainMenuItem::WalletConnect => Page::WalletConnect(WalletConnectPage::new()?),
            MainMenuItem::SignMessage => Page::SignMessage(SignMessagePage::new()?),
            MainMenuItem::SendMessage => Page::SendMessage(SendMessagePage::new()?),
            MainMenuItem::DevKeyInput => Page::DevKeyCapture(DevKeyCapturePage::default()),
            MainMenuItem::Shell => Page::Shell(ShellPage::default()),
            MainMenuItem::Config => Page::Config(ConfigPage::new()?),
        })
    }

    pub fn depends_on_current_account(&self) -> bool {
        match self {
            MainMenuItem::CompleteSetup
            | MainMenuItem::AddressBook
            | MainMenuItem::Networks
            | MainMenuItem::Accounts
            | MainMenuItem::WalletConnect
            | MainMenuItem::DevKeyInput
            | MainMenuItem::Config => false,

            MainMenuItem::Portfolio
            | MainMenuItem::SignMessage
            | MainMenuItem::SendMessage
            | MainMenuItem::Shell => true,
        }
    }

    pub fn only_on_developer_mode(&self) -> bool {
        match self {
            MainMenuItem::CompleteSetup
            | MainMenuItem::Portfolio
            | MainMenuItem::Accounts
            | MainMenuItem::AddressBook
            | MainMenuItem::Networks
            | MainMenuItem::WalletConnect
            | MainMenuItem::SignMessage
            | MainMenuItem::SendMessage
            | MainMenuItem::Shell
            | MainMenuItem::Config => false,
            MainMenuItem::DevKeyInput => true,
        }
    }

    pub fn get_menu(developer_mode: bool) -> crate::Result<Vec<MainMenuItem>> {
        let mut all_options: Vec<MainMenuItem> = MainMenuItem::iter().collect();

        #[cfg(feature = "demo")]
        all_options.remove(0);

        #[cfg(not(feature = "demo"))]
        {
            let temp_setup_page = CompleteSetupPage::new()?;
            if temp_setup_page.form.valid_count() == 0 {
                all_options.remove(0);
            }
        }

        let current_account_exists = Config::current_account().is_ok();
        let mut options = vec![];

        for option in all_options {
            if (!option.depends_on_current_account() || current_account_exists)
                && (!option.only_on_developer_mode() || developer_mode)
            {
                options.push(option);
            }
        }

        Ok(options)
    }
}

pub struct SharedState {
    pub online: Option<bool>,
    pub asset_manager: Arc<RwLock<AssetManager>>,
    pub price_manager: Arc<PriceManager>,
    pub recent_addresses: Option<Vec<Address>>,
    pub theme: Theme,
    pub config: Config,
    pub networks: Arc<NetworkStore>,
}

impl SharedState {
    pub fn assets_read(&self) -> crate::Result<Option<Vec<Asset>>> {
        let Ok(current_account) = self.try_current_account() else {
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
        Ok(self.config.get_current_account()?)
    }
}

struct AppAreas {
    title: Rect,
    middle: Rect,
    footer: Rect,
    menu: Rect,
    gap: Rect,
    body: Rect,
    popup: Rect,
}

#[derive(Clone, PartialEq)]
enum Focus {
    Menu,
    Body,
    Popup { on_close: Box<Focus> },
}

pub struct App {
    exit: bool,
    focus: Focus,
    pub main_menu: Select<MainMenuItem>,
    context: Vec<Page>,
    shared_state: SharedState,
    clipboard: Clipboard,
    copied_toast: Toast,
    opened_toast: Toast,

    fatal_error_popup: TextPopup,
    pub invite_popup: InvitePopup,
    #[cfg(feature = "demo")]
    demo_popup: TextPopup,

    tick_thread: Option<tokio::task::JoinHandle<()>>,
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
        let networks = Arc::new(NetworkStore::load_and_update()?);

        let config = Config::load()?;
        let theme_name = ThemeName::from_str(config.get_theme_name()).unwrap_or_default();
        let theme = Theme::new(theme_name);
        let mut app = Self {
            exit: false,
            focus: Focus::Menu,
            main_menu: Select::new(
                Some(MainMenuItem::get_menu(config.get_developer_mode())?),
                true,
            ),
            context: vec![Page::Assets(AssetsPage::new(None)?)],
            shared_state: SharedState {
                online: None,
                asset_manager: Arc::new(RwLock::new(AssetManager::default())),
                price_manager: Arc::new(PriceManager::new(&networks)?),
                recent_addresses: None,
                theme,
                config,
                networks,
            },
            clipboard: Clipboard::new().map_err(crate::Error::ArboardInitFailed)?,
            copied_toast: Toast::new("Copied"),
            opened_toast: Toast::new("Opened in browser"),

            fatal_error_popup: TextPopup::default().with_title("Fatal Error").with_note({
                "If you think this is a bug, please create issue at https://github.com/zemse/gm/new"
            }),
            invite_popup: InvitePopup::default(),
            #[cfg(feature = "demo")]
            demo_popup: TextPopup::default(),

            tick_thread: None,
            input_thread: None,
            refresh_prices_thread: None,
            assets_thread: None,
            recent_addresses_thread: None,
            helios_thread: None,

            #[cfg(feature = "demo")]
            demo_timer: Some(Instant::now()),
        };
        app.update_focus(Focus::Menu);

        Ok(app)
    }

    pub async fn run(&mut self, pre_events: Option<Vec<AppEvent>>) -> crate::Result<()> {
        let (event_tr, event_rc) = mpsc::channel::<AppEvent>();
        let shutdown = CancellationToken::new();
        let mut terminal = ratatui::init();
        execute!(stdout(), EnableMouseCapture)?;

        self.init_threads(&event_tr, &shutdown);

        #[cfg(feature = "demo")]
        self.demo_popup.set_text(demo_text().to_string(), true);

        if let Some(events) = pre_events {
            let area = self.draw(&mut terminal).map_err(crate::Error::Draw)?;
            for event in events {
                self.handle_event(event, area, &event_tr, &shutdown)
                    .await
                    .unwrap_or_else(|e| {
                        self.fatal_error_popup.set_text(e.to_string(), true);
                    })
            }
        }

        while !self.exit {
            let area = self.draw(&mut terminal).map_err(crate::Error::Draw)?;

            self.handle_event(event_rc.recv()?, area, &event_tr, &shutdown)
                .await
                .unwrap_or_else(|e| self.fatal_error_popup.set_text(e.to_string(), true));
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
        let tr_tick = tr.clone();
        let shutdown_signal = sd.clone();
        self.tick_thread = Some(tokio::spawn(async move {
            start_ticking(tr_tick, shutdown_signal).await;
        }));

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

        let theme_name = ThemeName::from_str(config.get_theme_name()).unwrap_or_default();
        let theme = Theme::new(theme_name);
        self.shared_state.theme = theme;

        self.main_menu
            .update_list(Some(MainMenuItem::get_menu(config.get_developer_mode())?));

        self.shared_state.config = config;
        self.shared_state.networks = Arc::new(NetworkStore::load()?);

        for page in &mut self.context {
            page.reload(&self.shared_state)?;
        }

        Ok(())
    }

    async fn process_result(
        &mut self,
        mut result: PostHandleEventActions,
    ) -> crate::Result<(bool, bool, bool)> {
        if let Some((text, tip_position)) = result.take_clipboard_request() {
            self.clipboard
                .set_text(text)
                .map_err(crate::Error::ArboardSetText)?;

            if let Some(tip_position) = tip_position {
                self.copied_toast.show(tip_position, Duration::from_secs(1));
            }
        }

        if let Some((url, tip_position)) = result.take_url_request() {
            open::that(url.to_string()).map_err(crate::Error::OpenThat)?;

            if let Some(tip_position) = tip_position {
                self.opened_toast.show(tip_position, Duration::from_secs(2));
            }
        }

        if result.get_page_pop() {
            self.context.pop();
        }
        if result.get_page_pop_all() {
            self.context.clear();
        }
        if result.get_reload() {
            self.reload()?;
            if let Some(page) = self.context.last_mut() {
                page.reload(&self.shared_state)?;
            }
        }
        if result.get_refresh_assets() {
            // TODO restart the assets thread to avoid the delay
            if let Ok(account) = Config::current_account() {
                self.shared_state.assets_mut()?.clear_data_for(account);
            }
        }
        self.context.extend(result.get_page_inserts_owned());
        Ok((
            result.get_ignore_esc(),
            result.get_ignore_ctrlc(),
            result.get_ignore_left(),
        ))
    }

    fn update_popup_focus(&mut self, to_popup: bool) {
        if to_popup {
            if !matches!(self.focus, Focus::Popup { .. }) {
                let old_focus = self.focus.clone();

                let new_focus = Focus::Popup {
                    on_close: Box::new(old_focus),
                };

                self.update_focus(new_focus);
            }
        } else if let Focus::Popup { on_close } = &self.focus {
            let old_focus = on_close.as_ref().clone();
            self.update_focus(old_focus);
        }
    }

    fn update_focus(&mut self, new_focus: Focus) {
        self.focus = new_focus;

        if self.context.is_empty() {
            self.focus = Focus::Menu;
        } else if self.focus == Focus::Menu {
            if let Some(page) = self.context.last_mut() {
                page.set_focus(false);
            }
            self.main_menu.set_focus(true);
        } else if self.focus == Focus::Body {
            if let Some(page) = self.context.last_mut() {
                page.set_focus(true);
            }
            self.main_menu.set_focus(false);
        } else {
            if let Some(page) = self.context.last_mut() {
                page.set_focus(false);
            }
            self.main_menu.set_focus(false);
        }
    }

    async fn handle_event(
        &mut self,
        event: AppEvent,
        area: Rect,
        tr: &mpsc::Sender<AppEvent>,
        sd: &CancellationToken,
    ) -> crate::Result<()> {
        let areas = self.get_areas(area);

        #[cfg(feature = "demo")]
        if let Some(demo_timer) = self.demo_timer {
            use crate::demo::DEMO_2_DELAY;

            if demo_timer.elapsed() >= DEMO_2_DELAY {
                self.demo_timer = None;
                self.demo_popup.set_text(demo_text_2().to_string(), true);
            }
        }

        #[cfg(feature = "demo")]
        let demo_popup_shown = self.demo_popup.is_open();
        #[cfg(not(feature = "demo"))]
        let demo_popup_shown = false;

        let fatal_error_popup_open = self.fatal_error_popup.is_open();

        let is_invite_popup_open = self.invite_popup.is_open();

        self.update_popup_focus(demo_popup_shown || fatal_error_popup_open || is_invite_popup_open);

        // Update state based on events
        match &event {
            AppEvent::PricesUpdate => {
                // The prices have already been updated in the PriceManager store in shared_state
                self.set_online(tr, sd)?;
            }
            AppEvent::PricesUpdateError(error) => {
                if error.is_connect() {
                    // ETH Price is the main API for understanding if we are connected to internet
                    self.set_offline().await;
                } else {
                    self.fatal_error_popup.set_text(error.to_string(), true);
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
                    self.fatal_error_popup.set_text(error.to_string(), true);
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
                self.fatal_error_popup.set_text(error.clone(), true);
            }

            AppEvent::RecentAddressesUpdate(addresses) => {
                self.shared_state.recent_addresses = Some(addresses.clone());
            }
            AppEvent::RecentAddressesUpdateError(error) => {
                self.fatal_error_popup.set_text(error.to_string(), true);
            }

            // Candles API
            AppEvent::CandlesUpdateError(error) => {
                self.fatal_error_popup.set_text(error.to_string(), true);
            }

            // Transaction API
            AppEvent::TxError(error) => self.fatal_error_popup.set_text(error.clone(), true),

            AppEvent::WalletConnectError(_, error) => {
                self.fatal_error_popup.set_text(error.clone(), true);
            }

            AppEvent::InviteError(error) => {
                self.fatal_error_popup.set_text(error.clone(), true);
            }

            _ => {}
        }

        let mut actions = PostHandleEventActions::default();

        self.copied_toast.handle_event(event.widget_event());
        self.opened_toast.handle_event(event.widget_event());

        // Handle the event in the relavent component
        if fatal_error_popup_open {
            self.fatal_error_popup
                .handle_event::<PostHandleEventActions>(
                    event.input_event(),
                    areas.popup,
                    &mut actions,
                );
        } else if is_invite_popup_open {
            self.invite_popup
                .handle_event(&event, tr, &self.shared_state, &mut actions)?
        } else if demo_popup_shown {
            #[cfg(not(feature = "demo"))]
            unreachable!();
            #[cfg(feature = "demo")]
            self.demo_popup.handle_event::<PostHandleEventActions>(
                event.input_event(),
                areas.popup,
                &mut actions,
            );
        } else {
            let is_key_event = event.key_event().is_some();
            let mut body_area = areas.body;

            if self.context.len() > 1 {
                body_area = body_area.margin_top(2);
            }

            // If focus is on body, handle all events. However if focus is not on body
            // then handle all but key events. This allows to handle mouse clicks and widget updates.
            if (self.focus == Focus::Body || !is_key_event) && self.context.last().is_some() {
                let page = self.context.last_mut().unwrap();
                let r =
                    page.handle_event(&event, body_area, areas.popup, tr, sd, &self.shared_state)?;
                actions.merge(r);
            }

            // If focus is on menu, handle all events. However if focus is not on menu
            // then handle all but key events. This allows to handle mouse clicks.
            if self.focus == Focus::Menu || !is_key_event {
                match self
                    .main_menu
                    .handle_event(event.input_event(), areas.menu)?
                {
                    Some(SelectEvent::Select(item)) => {
                        let mut page = item.get_page(&self.shared_state)?;
                        page.set_focus(true);
                        actions.page_pop_all();
                        actions.page_insert(page);

                        self.update_focus(Focus::Body);
                    }
                    Some(SelectEvent::Hover { on_list_area }) => {
                        if on_list_area {
                            self.update_focus(Focus::Menu);
                        } else {
                            // TODO this is not right solution. we should update focus to body if mouse hovers on body
                            self.update_focus(Focus::Body);
                        }
                    }
                    None => {}
                }
            }
        };

        let (ignore_esc, ignore_ctrlc, ignore_left) = self.process_result(actions).await?;

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
                            self.fatal_error_popup
                                .set_text("test error".to_string(), true);
                        }
                        if char == 't' && key_event.modifiers == KeyModifiers::CONTROL {
                            self.context.push(Page::Trade(TradePage::default()));
                        }
                    }
                    KeyCode::Right => {
                        if self.focus == Focus::Menu {
                            self.update_focus(Focus::Body);
                            if let Some(page) = self.context.last_mut() {
                                page.set_cursor(self.main_menu.cursor());
                            }
                        }
                    }
                    KeyCode::Left => {
                        if self.focus == Focus::Body
                            && (!ignore_left || key_event.modifiers.contains(KeyModifiers::SHIFT))
                        {
                            if let Some(cursor) = self
                                .context
                                .last()
                                .and_then(|page| page.get_cursor())
                                .or(self.main_menu.hover_cursor())
                            {
                                self.main_menu.set_cursor(cursor);
                            }
                            self.update_focus(Focus::Menu);
                        }
                    }
                    KeyCode::Esc => {
                        if !ignore_esc {
                            if self.context.len() == 1 && self.focus != Focus::Menu {
                                self.update_focus(Focus::Menu);
                            } else {
                                let page = self.context.pop();

                                if self.context.is_empty() {
                                    self.exit = true;
                                }

                                if let Some(mut page) = page {
                                    page.exit_threads().await;

                                    if self.context.is_empty() {
                                        self.context.push(page); // Push it back to prevent empty screen while exiting
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        };

        Ok(())
    }

    fn get_areas(&self, area: Rect) -> AppAreas {
        let [title_area, middle_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .areas(area);
        let [menu_area, gap_area, body_area] = Layout::horizontal([
            Constraint::Length(14),
            Constraint::Length(1),
            Constraint::Min(2),
        ])
        .areas(middle_area.block_inner());

        AppAreas {
            title: title_area,
            middle: middle_area,
            footer: footer_area,
            menu: menu_area,
            gap: gap_area.expand_vertical(1),
            body: body_area,
            popup: {
                let diff = |num: u16| num.mul(1).saturating_div(8).max(2);
                let width_diff = diff(area.width);
                let height_diff = diff(area.height);

                Rect {
                    width: area.width - 2 * width_diff,
                    height: area.height - 2 * height_diff,
                    x: area.x + width_diff,
                    y: area.y + height_diff,
                }
            },
        }
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

        let areas = self.get_areas(area);

        Title.render_component(areas.title, areas.popup, buf, &self.shared_state);

        if self.shared_state.theme.boxed() {
            // Render border of the canvas
            Block::bordered()
                .style(self.shared_state.theme.style_dim())
                .border_type(self.shared_state.theme.border_type)
                .render(areas.middle, buf);

            // Render the Middle stick
            let height_inner = areas.gap.height - 2;
            let mut gap_area = areas.gap;
            "┬".render(gap_area, buf);
            for _ in 0..height_inner {
                gap_area = gap_area.margin_top(1);
                "│".render(gap_area, buf);
            }
            gap_area = gap_area.margin_top(1);
            "┴".render(gap_area, buf);
        }

        // Render Main Menu on the Left side
        self.main_menu
            .render(areas.menu, buf, &self.shared_state.theme);

        let mut body_area = areas.body;
        if self.context.len() > 1 {
            let mut nav_area = areas.body.change_height(1);
            body_area = body_area.margin_top(2);

            let names = self
                .context
                .iter()
                .map(|page| page.name())
                .collect::<Vec<Cow<'static, str>>>();

            for (i, name) in names.iter().enumerate() {
                let name_size = name.len();
                let is_last = i == names.len() - 1;

                Span::raw(name.to_string())
                    .style(if is_last {
                        self.shared_state.theme.style()
                    } else {
                        self.shared_state.theme.style_dim()
                    })
                    .render(nav_area, buf);
                nav_area = nav_area.margin_left(name_size as u16);

                if !is_last {
                    Span::raw(" / ")
                        .style(self.shared_state.theme.style_dim())
                        .render(nav_area, buf);
                    nav_area = nav_area.margin_left(3);
                }
            }
        }

        if let Some(page) = self.current_page() {
            page.render_component(body_area, areas.popup, buf, &self.shared_state);
        }

        Footer {
            exit: &self.exit,
            is_main_menu: &true, // TODO improve
        }
        .render(areas.footer, buf, &self.shared_state.theme);

        self.invite_popup
            .render(areas.popup, buf, &self.shared_state);

        self.fatal_error_popup
            .render(areas.popup, buf, &self.shared_state.theme.error_popup());

        #[cfg(feature = "demo")]
        self.demo_popup
            .render(areas.popup, buf, &self.shared_state.theme.popup());

        self.copied_toast.render(buf, &self.shared_state.theme);
        self.opened_toast.render(buf, &self.shared_state.theme);
    }
}
