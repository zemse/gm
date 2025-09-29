use std::sync::mpsc;

use gm_utils::config::Config;
use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind},
    layout::Rect,
    widgets::Widget,
};
use strum::{Display, EnumIter, IntoEnumIterator};
use tokio_util::sync::CancellationToken;

use super::{
    account::AccountPage, address_book::AddressBookPage, assets::AssetsPage,
    complete_setup::CompleteSetupPage, config::ConfigPage, dev_key_capture::DevKeyCapturePage,
    send_message::SendMessagePage, sign_message::SignMessagePage, walletconnect::WalletConnectPage,
    Page,
};
use crate::pages::{network::NetworkPage, shell::ShellPage};
use crate::{
    app::SharedState,
    events::Event,
    traits::{Actions, Component},
};
use gm_ratatui_extra::{
    thematize::Thematize,
    widgets::{cursor::Cursor, select::Select},
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
    pub fn get_page(&self) -> crate::Result<Page> {
        Ok(match self {
            MainMenuItem::CompleteSetup => Page::CompleteSetup(CompleteSetupPage::new()?),
            MainMenuItem::Portfolio => Page::Assets(AssetsPage::default()),
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

#[derive(Debug)]
pub struct MainMenuPage {
    pub cursor: Cursor,
    pub list: Vec<MainMenuItem>,
}

impl MainMenuPage {
    pub fn new(developer_mode: bool) -> crate::Result<Self> {
        Ok(Self {
            list: MainMenuItem::get_menu(developer_mode)?,
            cursor: Cursor::default(),
        })
    }

    pub fn get_focussed_item(&self) -> &MainMenuItem {
        self.list
            .get(self.cursor.current)
            .expect("Invalid cursor position in MainMenuPage")
    }

    pub fn set_focussed_item(&mut self, item: MainMenuItem) {
        if let Some((index, _)) = self.list.iter().enumerate().find(|(_, i)| **i == item) {
            self.cursor.current = index;
        }
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
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let cursor_max = self.list.len();
        self.cursor.handle(event.key_event(), cursor_max);

        let mut result = Actions::default();
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
            focus_style: shared_state.theme.select_focused(),
        }
        .render(area, buf);

        area
    }
}
