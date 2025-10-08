use std::mem;

use gm_ratatui_extra::act::Act;

use crate::pages::Page;

#[derive(Default, Debug)]
pub struct PostHandleEventActions {
    // Enable if current page wants to handle the [ESC] key.
    ignore_esc: bool,
    // Ignore left arrow key presses which would move focus to menu
    ignore_left: bool,
    // Ignore [CTRL+C] key presses which would quit app
    ignore_ctrlc: bool,
    // Remove the current page from the context stack.
    page_pop: bool,
    // Remove all pages from the context stack.
    page_pop_all: bool,
    // Page to insert into the context stack.
    page_inserts: Vec<Page>,
    // Regenerate the data for the current page, this is used when we expect
    // that the external state is updated and we need to reflect that in the UI.
    reload: bool,
    // Clears data for assets and refetches them.
    refresh_assets: bool,
}

impl Act for PostHandleEventActions {
    fn merge(&mut self, other: PostHandleEventActions) {
        self.ignore_esc |= other.ignore_esc;
        self.ignore_left |= other.ignore_left;
        self.ignore_ctrlc |= other.ignore_ctrlc;
        self.page_pop |= other.page_pop;
        self.page_pop_all |= other.page_pop_all;
        self.page_inserts.extend(other.page_inserts);
        self.reload |= other.reload;
        self.refresh_assets |= other.refresh_assets;
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
}

impl PostHandleEventActions {
    pub fn ignore_ctrlc(&mut self) {
        self.ignore_ctrlc = true;
    }

    pub fn page_pop(&mut self) {
        self.page_pop = true;
    }

    pub fn page_pop_all(&mut self) {
        self.page_pop_all = true;
    }

    pub fn page_insert(&mut self, page: Page) {
        self.page_inserts.push(page);
    }

    pub fn reload(&mut self) {
        self.reload = true;
    }

    pub fn refresh_assets(&mut self) {
        self.refresh_assets = true;
    }

    pub fn get_ignore_esc(&self) -> bool {
        self.ignore_esc
    }

    pub fn get_ignore_left(&self) -> bool {
        self.ignore_left
    }

    pub fn get_ignore_ctrlc(&self) -> bool {
        self.ignore_ctrlc
    }

    pub fn get_page_pop(&self) -> bool {
        self.page_pop
    }

    pub fn get_page_pop_all(&self) -> bool {
        self.page_pop_all
    }

    pub fn get_page_inserts_owned(&mut self) -> Vec<Page> {
        mem::take(&mut self.page_inserts)
    }

    pub fn get_reload(&self) -> bool {
        self.reload
    }

    pub fn get_refresh_assets(&self) -> bool {
        self.refresh_assets
    }
}
