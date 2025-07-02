use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
};

use crate::tui::{
    theme::Theme,
    traits::{HandleResult, RectUtil},
    Event,
};

use super::{button::Button, popup::Popup, text_scroll::TextScroll};

pub struct ConfirmPopup {
    title: &'static str,
    text: TextScroll,
    confirm_button_label: &'static str,
    cancel_button_label: &'static str,
    open: bool,
    button_cursor: bool, // is cursor on the confirm button?
}

impl ConfirmPopup {
    pub fn new(
        title: &'static str,
        text: String,
        confirm_button_label: &'static str,
        cancel_button_label: &'static str,
    ) -> Self {
        Self {
            title,
            text: TextScroll::new(text),
            confirm_button_label,
            cancel_button_label,
            open: false,
            button_cursor: false,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    // Opens the popup with the fresh items.
    pub fn open(&mut self) {
        self.open = true;
        self.button_cursor = false;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn text_mut(&mut self) -> &mut String {
        &mut self.text.text
    }

    pub fn handle_event<F1, F2>(
        &mut self,
        event: &crate::tui::Event,
        area: Rect,
        mut on_confirm: F1,
        mut on_cancel: F2,
    ) -> crate::Result<HandleResult>
    where
        F1: FnMut() -> crate::Result<()>,
        F2: FnMut() -> crate::Result<()>,
    {
        let mut result = HandleResult::default();

        if self.open {
            let text_area = Popup::inner_area(area).block_inner().margin_down(3);
            self.text.handle_event(event, text_area)?;

            if let Event::Input(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Left => {
                            self.button_cursor = false;
                        }
                        KeyCode::Right => {
                            self.button_cursor = true;
                        }
                        KeyCode::Enter => {
                            if self.button_cursor {
                                on_confirm()?;
                            } else {
                                on_cancel()?;
                            }
                            self.close();
                        }
                        KeyCode::Esc => {
                            on_cancel()?;
                            self.close();
                        }
                        _ => {}
                    }
                }
            }

            result.esc_ignores = 1;
        }

        Ok(result)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            Popup.render(area, buf, &theme);

            let inner_area = Popup::inner_area(area);
            let block = Block::bordered().title(self.title);
            let block_inner_area = block.inner(inner_area);
            block.render(inner_area, buf);

            let [text_area, button_area] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(3)])
                    .areas(block_inner_area);

            self.text.render(text_area, buf);

            let [left_area, right_area] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(button_area);

            Button {
                focus: !self.button_cursor,
                label: self.cancel_button_label,
            }
            .render(left_area, buf, &theme);

            Button {
                focus: self.button_cursor,
                label: self.confirm_button_label,
            }
            .render(right_area, buf, &theme);
        }
    }
}
