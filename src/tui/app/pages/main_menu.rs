use std::sync::{atomic::AtomicBool, mpsc, Arc};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{layout::Rect, widgets::Widget};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{
    disk::Config,
    tui::{
        app::{widgets::select::Select, SharedState},
        events::Event,
        traits::{Component, HandleResult},
    },
    utils::cursor::Cursor,
};

use super::{
    account::AccountPage, address_book::AddressBookPage, assets::AssetsPage, config::ConfigPage,
    dev_key_capture::DevKeyCapturePage, send_message::SendMessagePage, setup::SetupPage,
    sign_message::SignMessagePage, walletconnect::WalletConnectPage, Page,
};

#[derive(Display, EnumIter)]
pub enum MainMenuItem {
    Setup,
    Portfolio,
    Accounts,
    AddressBook,
    WalletConnect,
    SignMessage,
    SendMessage,
    DevKeyInput,
    Config,
}

impl MainMenuItem {
    pub fn get_page(&self) -> crate::Result<Page> {
        Ok(match self {
            MainMenuItem::Setup => Page::Setup(SetupPage::new()?),
            MainMenuItem::Portfolio => Page::Assets(AssetsPage::default()),
            MainMenuItem::Accounts => Page::Account(AccountPage::new()?),
            MainMenuItem::AddressBook => Page::AddressBook(AddressBookPage::new()?),
            MainMenuItem::WalletConnect => Page::WalletConnect(WalletConnectPage::new()?),
            MainMenuItem::SignMessage => Page::SignMessage(SignMessagePage::new()?),
            MainMenuItem::SendMessage => Page::SendMessage(SendMessagePage::new()?),
            MainMenuItem::DevKeyInput => Page::DevKeyCapture(DevKeyCapturePage::default()),
            MainMenuItem::Config => Page::Config(ConfigPage::new()?),
        })
    }

    pub fn depends_on_current_account(&self) -> bool {
        match self {
            MainMenuItem::Setup
            | MainMenuItem::AddressBook
            | MainMenuItem::Accounts
            | MainMenuItem::WalletConnect
            | MainMenuItem::DevKeyInput
            | MainMenuItem::Config => false,

            MainMenuItem::Portfolio | MainMenuItem::SignMessage | MainMenuItem::SendMessage => true,
        }
    }

    pub fn only_on_developer_mode(&self) -> bool {
        match self {
            MainMenuItem::Setup
            | MainMenuItem::Portfolio
            | MainMenuItem::Accounts
            | MainMenuItem::AddressBook
            | MainMenuItem::WalletConnect
            | MainMenuItem::SignMessage
            | MainMenuItem::SendMessage
            | MainMenuItem::Config => false,
            MainMenuItem::DevKeyInput => true,
        }
    }

    pub fn get_menu(developer_mode: bool) -> crate::Result<Vec<MainMenuItem>> {
        let mut all_options: Vec<MainMenuItem> = MainMenuItem::iter().collect();

        let temp_setup_page = SetupPage::new()?;
        if temp_setup_page.form.visible_count() == 0 {
            all_options.remove(0);
        }

        let current_account_exists = Config::current_account()?.is_some();
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

pub struct MainMenuPage {
    pub cursor: Cursor,
    pub list: Vec<MainMenuItem>,
}

impl MainMenuPage {
    pub fn get_focussed_item(&self) -> &MainMenuItem {
        self.list
            .get(self.cursor.current)
            .expect("Invalid cursor position in MainMenuPage")
    }
}

// TODO I am trying to hide the dev key capture page
impl MainMenuPage {
    pub fn new(developer_mode: bool) -> crate::Result<Self> {
        Ok(Self {
            list: MainMenuItem::get_menu(developer_mode)?,
            cursor: Cursor::default(),
        })
    }
}

impl Component for MainMenuPage {
    fn reload(&mut self, shared_state: &SharedState) -> crate::Result<()> {
        let fresh = Self::new(shared_state.developer_mode)?;
        self.list = fresh.list;
        if self.cursor.current >= self.list.len() {
            self.cursor.current = self.list.len() - 1;
        }
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        let cursor_max = self.list.len();
        self.cursor.handle(event, cursor_max);

        let mut result = HandleResult::default();
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                #[allow(clippy::single_match)]
                match key_event.code {
                    KeyCode::Enter => {
                        let mut page = self.list[self.cursor.current].get_page()?;
                        page.set_focus(true);
                        result.page_inserts.push(page);
                    }
                    _ => {}
                }
            }
        };

        Ok(result)
    }

    fn render_component(
        &self,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
        shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        Select {
            list: &self.list,
            cursor: &self.cursor,
            focus: true,
            focus_style: shared_state.theme.select(),
        }
        .render(area, buf);

        area
    }
}
