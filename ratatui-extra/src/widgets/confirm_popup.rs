use std::mem;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
};

use crate::{
    act::Act,
    button::ButtonResult,
    extensions::{EventExt, RectExt, ThemedWidget},
    thematize::Thematize,
};

use super::{button::Button, popup::Popup, text_scroll::TextScroll};

struct Areas {
    title_area: Rect,
    body_area: Rect,
    confirm_button_area: Rect,
    cancel_button_area: Rect,
}

pub enum ConfirmResult {
    Confirmed,
    Canceled,
}

#[derive(Debug)]
pub struct ConfirmPopup {
    title: &'static str,
    text: TextScroll,
    open: bool,
    confirm_button: Button,
    cancel_button: Button,
    is_confirm_focused: bool,
    initial_cursor_on_confirm: bool,
}

impl ConfirmPopup {
    pub fn new(
        title: &'static str,
        text: String,
        confirm_button_label: &'static str,
        cancel_button_label: &'static str,
        initial_cursor_on_confirm: bool,
    ) -> Self {
        Self {
            title,
            text: TextScroll::new(text, true),
            confirm_button: Button::new(confirm_button_label).with_success_kind(true),
            cancel_button: Button::new(cancel_button_label).with_success_kind(true),
            open: false,
            is_confirm_focused: initial_cursor_on_confirm,
            initial_cursor_on_confirm,
        }
    }

    pub fn open_already(mut self) -> Self {
        self.open();
        self
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    // Opens the popup with the fresh items.
    pub fn open(&mut self) {
        self.open = true;
        self.is_confirm_focused = self.initial_cursor_on_confirm;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn text(&self) -> &String {
        &self.text.text
    }

    pub fn into_text_scroll(&mut self) -> TextScroll {
        mem::take(&mut self.text)
    }

    pub fn text_mut(&mut self) -> &mut String {
        &mut self.text.text
    }

    pub fn handle_event<A>(
        &mut self,
        input_event: Option<&Event>,
        area: Rect,
        actions: &mut A,
    ) -> Result<Option<ConfirmResult>, crate::Error>
    where
        A: Act,
    {
        let mut result = None;

        if self.open {
            actions.ignore_left();
            actions.ignore_right();

            let areas = self.get_areas(area);

            if let Some(input_event) = input_event {
                self.text
                    .handle_event(input_event.key_event(), areas.body_area);

                if let Some(button_event) = self.confirm_button.handle_event(
                    Some(input_event),
                    areas.confirm_button_area,
                    self.is_confirm_focused,
                ) {
                    match button_event {
                        ButtonResult::Pressed => {
                            result = Some(ConfirmResult::Confirmed);
                            self.open = false;
                        }
                        ButtonResult::HoverIn(is_focused) => {
                            if is_focused {
                                self.is_confirm_focused = true;
                            }
                        }
                    }
                }

                if let Some(button_event) = self.cancel_button.handle_event(
                    Some(input_event),
                    areas.cancel_button_area,
                    !self.is_confirm_focused,
                ) {
                    match button_event {
                        ButtonResult::Pressed => {
                            result = Some(ConfirmResult::Canceled);
                            self.open = false;
                        }
                        ButtonResult::HoverIn(is_focused) => {
                            if is_focused {
                                self.is_confirm_focused = false;
                            }
                        }
                    }
                }

                if let Event::Key(key_event) = input_event {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Left => {
                                self.is_confirm_focused = false;
                            }
                            KeyCode::Right => {
                                self.is_confirm_focused = true;
                            }
                            KeyCode::Esc => {
                                result = Some(ConfirmResult::Canceled);
                                self.close();
                            }
                            _ => {}
                        }
                    }
                }

                actions.ignore_esc();
            }
        }

        Ok(result)
    }

    fn get_areas(&self, area: Rect) -> Areas {
        let inner_area = Popup::inner_area(area);

        let [title_area, body_area, button_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .areas(inner_area);

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(button_area);
        Areas {
            title_area,
            body_area,
            confirm_button_area: right_area.button_center(self.confirm_button.label.len()),
            cancel_button_area: left_area.button_center(self.cancel_button.label.len()),
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            Popup.render(area, buf, &theme);
            Block::bordered().title(self.title).inner(area);

            let areas = self.get_areas(area);
            self.title.render(areas.title_area, buf);
            self.text.render(areas.body_area, buf, &theme);

            self.confirm_button.render(
                areas.confirm_button_area,
                buf,
                self.is_confirm_focused,
                &theme,
            );

            self.cancel_button.render(
                areas.cancel_button_area,
                buf,
                !self.is_confirm_focused,
                &theme,
            );
        }
    }
}
