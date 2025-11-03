use std::mem;

use gm_ratatui_extra::act::Act;
use ratatui::layout::Position;
use url::Url;

use crate::pages::Page;

/// Actions that the App should take after handling an event. This
/// is passed to all the components and they can modify it to indicate
/// what actions the App should take after the event is handled.
#[derive(Default, Debug)]
pub struct PostHandleEventActions {
    /// Enable if current page wants to handle the [ESC] key.
    ignore_esc: bool,
    /// Ignore left arrow key presses which would move focus to menu
    ignore_left: bool,
    /// Ignore [CTRL+C] key presses which would quit app
    ignore_ctrlc: bool,
    /// Remove the current page from the context stack.
    page_pop: bool,
    /// Remove all pages from the context stack.
    page_pop_all: bool,
    /// Page to insert into the context stack.
    page_inserts: Vec<Page>,
    /// Regenerate the data for the current page, this is used when we expect
    /// that the external state is updated and we need to reflect that in the UI.
    reload: bool,
    /// Clears data for assets and refetches them.
    refresh_assets: bool,
    /// Copy to clipboard request
    copy_to_clipboard: Option<(String, Option<Position>)>,
    /// Open URL request
    open_url: Option<(Url, Option<Position>)>,
    /// Error
    error: Option<crate::Error>,
}

impl Act for PostHandleEventActions {
    // TODO remove
    fn merge(&mut self, other: PostHandleEventActions) {
        self.ignore_esc |= other.ignore_esc;
        self.ignore_left |= other.ignore_left;
        self.ignore_ctrlc |= other.ignore_ctrlc;
        self.page_pop |= other.page_pop;
        self.page_pop_all |= other.page_pop_all;
        self.page_inserts.extend(other.page_inserts);
        self.reload |= other.reload;
        self.refresh_assets |= other.refresh_assets;
        self.copy_to_clipboard = other.copy_to_clipboard.or(self.copy_to_clipboard.take());
        self.open_url = other.open_url.or(self.open_url.take());
        self.error = other.error.or(self.error.take());
    }

    fn ignore_esc(&mut self) {
        self.ignore_esc = true;
    }

    fn ignore_left(&mut self) {
        self.ignore_left = true;
    }

    fn ignore_right(&mut self) {
        // we don't care about right arrow key presses
    }

    fn is_esc_ignored(&self) -> bool {
        self.ignore_esc
    }

    fn copy_to_clipboard(&mut self, text: String, tip_position: Option<Position>) {
        self.copy_to_clipboard = Some((text, tip_position));
    }

    fn open_url(&mut self, url: Url, tip_position: Option<Position>) {
        self.open_url = Some((url, tip_position));
    }
}

impl PostHandleEventActions {
    /// Enable if current page wants to handle the [CTRL+C] key.
    pub fn ignore_ctrlc(&mut self) {
        self.ignore_ctrlc = true;
    }

    /// Causes the page on the front to be removed from the context stack.
    /// It appears as a "back" action to the user.
    pub fn page_pop(&mut self) {
        self.page_pop = true;
    }

    /// Causes all pages to be removed from the context stack.
    /// A page_insert should be used after this to avoid empty stack.
    pub fn page_pop_all(&mut self) {
        self.page_pop_all = true;
    }

    /// Inserts a page on top of the context stack.
    /// The inserted page will be the new active page.
    pub fn page_insert(&mut self, page: Page) {
        self.page_inserts.push(page);
    }

    /// Causes the App to reload all data like config, networks, also
    /// reload function in the active pages is called.
    pub fn reload(&mut self) {
        self.reload = true;
    }

    /// Causes the App to refetch assets urgently, should be used when
    /// we expect the balances to change.
    pub fn refresh_assets(&mut self) {
        self.refresh_assets = true;
    }

    /// Sets an error to be displayed by the App.
    pub fn set_error(&mut self, error: crate::Error) {
        self.error = Some(error);
    }

    /// Getter for ignore_esc, if this is true the App should not
    /// handle the [ESC] key.
    pub fn get_ignore_esc(&self) -> bool {
        self.ignore_esc
    }

    /// Getter for ignore_left, if this is true the App should not
    /// handle the left arrow key.
    pub fn get_ignore_left(&self) -> bool {
        self.ignore_left
    }

    /// Getter for ignore_ctrlc, if this is true the App should not
    /// handle the [CTRL+C] key.
    pub fn get_ignore_ctrlc(&self) -> bool {
        self.ignore_ctrlc
    }

    /// Getter for page_pop, if this is true the App should pop the current page.
    pub fn get_page_pop(&self) -> bool {
        self.page_pop
    }

    /// Getter for page_pop_all, if this is true the App should pop all pages on stack.
    pub fn get_page_pop_all(&self) -> bool {
        self.page_pop_all
    }

    /// Getter for page_inserts which takes the ownership of the vector.
    /// App uses this to insert pages on top of the context stack.
    pub fn get_page_inserts_owned(&mut self) -> Vec<Page> {
        mem::take(&mut self.page_inserts)
    }

    /// Getter for reload, if this is true the App should reload all data.
    pub fn get_reload(&self) -> bool {
        self.reload
    }

    /// Getter for refresh_assets, if this is true the App should refetch balances urgently.
    pub fn get_refresh_assets(&self) -> bool {
        self.refresh_assets
    }

    pub fn take_clipboard_request(&mut self) -> Option<(String, Option<Position>)> {
        self.copy_to_clipboard.take()
    }

    pub fn take_url_request(&mut self) -> Option<(Url, Option<Position>)> {
        self.open_url.take()
    }

    pub fn take_error(&mut self) -> Option<crate::Error> {
        self.error.take()
    }
}
