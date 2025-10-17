use std::mem;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
};

use crate::{
    act::Act,
    button::ButtonResult,
    extensions::{RectExt, ThemedWidget},
    popup::PopupWidget,
    thematize::Thematize,
};

use super::{button::Button, popup::Popup, text_interactive::TextInteractive};

struct Areas {
    text_area: Rect,
    confirm_button_area: Rect,
    cancel_button_area: Rect,
}

pub enum ConfirmResult {
    Confirmed,
    Canceled,
}

#[derive(Debug)]
pub struct ConfirmPopup {
    popup: Popup,
    text: TextInteractive,
    confirm_button: Button,
    cancel_button: Button,
    is_confirm_focused: bool,
    initial_cursor_on_confirm: bool,
}

impl PopupWidget for ConfirmPopup {
    fn get_popup(&self) -> &Popup {
        &self.popup
    }

    fn get_popup_mut(&mut self) -> &mut Popup {
        &mut self.popup
    }

    // Overrides the default open to also reset the focus to the confirm button if need
    fn open(&mut self) {
        self.popup.open();
        self.is_confirm_focused = self.initial_cursor_on_confirm;
    }
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
            popup: Popup::default().with_title(title),
            text: TextInteractive::default().with_text(text),
            confirm_button: Button::new(confirm_button_label).with_success_kind(true),
            cancel_button: Button::new(cancel_button_label).with_success_kind(true),
            is_confirm_focused: initial_cursor_on_confirm,
            initial_cursor_on_confirm,
        }
    }

    pub fn text(&self) -> &str {
        self.text.text()
    }

    pub fn into_text_scroll(&mut self) -> TextInteractive {
        mem::take(&mut self.text)
    }

    pub fn set_text(&mut self, text: String, scroll_to_top: bool) {
        self.text.set_text(text, scroll_to_top);
    }

    pub fn handle_event<A>(
        &mut self,
        input_event: Option<&Event>,
        popup_area: Rect,
        actions: &mut A,
    ) -> Result<Option<ConfirmResult>, crate::Error>
    where
        A: Act,
    {
        let mut result = None;

        if self.is_open() {
            actions.ignore_left();
            actions.ignore_right();

            let areas = self.get_areas(popup_area);

            if let Some(input_event) = input_event {
                self.text
                    .handle_event(Some(input_event), areas.text_area, actions);

                if let Some(button_event) = self.confirm_button.handle_event(
                    Some(input_event),
                    areas.confirm_button_area,
                    !self.text.is_focused() && self.is_confirm_focused,
                ) {
                    match button_event {
                        ButtonResult::Pressed => {
                            result = Some(ConfirmResult::Confirmed);
                            self.close();
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
                    !self.text.is_focused() && !self.is_confirm_focused,
                ) {
                    match button_event {
                        ButtonResult::Pressed => {
                            result = Some(ConfirmResult::Canceled);
                            self.close();
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
                                if !actions.is_esc_ignored() {
                                    result = Some(ConfirmResult::Canceled);
                                    self.close();
                                }
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

    fn get_areas(&self, popup_area: Rect) -> Areas {
        let body_area = self.body_area(popup_area);

        // Split the body area into text area and button area
        let [text_area, button_area] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(body_area);

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(button_area);
        Areas {
            text_area,
            confirm_button_area: right_area.button_center(self.confirm_button.label.len()),
            cancel_button_area: left_area.button_center(self.cancel_button.label.len()),
        }
    }

    pub fn render(&self, popup_area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();
            let areas = self.get_areas(popup_area);

            self.popup.render(popup_area, buf, &theme);
            self.text.render(areas.text_area, buf, &theme);

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
