use std::{collections::HashMap, marker::PhantomData};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
};
use strum::IntoEnumIterator;

use super::{button::Button, input_box::InputBox};
use crate::act::Act;
use crate::extensions::{RectExt, WidgetHeight};
use crate::widgets::scroll_bar::CustomScrollBar;
use crate::{thematize::Thematize, widgets::filter_select_popup::FilterSelectPopup};
use gm_utils::text::split_string;

pub trait FormItemIndex {
    fn index(self) -> usize;
}

#[derive(Clone, Debug)]
pub enum FormWidget {
    Heading(&'static str),
    StaticText(&'static str),
    InputBox {
        label: &'static str,
        text: String,
        empty_text: Option<&'static str>,
        currency: Option<String>,
    },
    BooleanInput {
        label: &'static str,
        value: bool,
    },
    DisplayBox {
        label: &'static str,
        text: String,
        empty_text: Option<&'static str>,
    },
    SelectInput {
        label: &'static str,
        text: String,
        empty_text: Option<&'static str>,
        popup: FilterSelectPopup<String>,
    },
    Button {
        label: &'static str,
    },
    DisplayText(String),
    ErrorText(String),
}

impl FormWidget {
    pub fn label(&self) -> Option<&'static str> {
        match self {
            FormWidget::InputBox { label, .. } => Some(label),
            FormWidget::DisplayBox { label, .. } => Some(label),
            FormWidget::BooleanInput { label, .. } => Some(label),
            FormWidget::Button { label } => Some(label),
            FormWidget::SelectInput { label, .. } => Some(label),
            FormWidget::Heading(_)
            | FormWidget::StaticText(_)
            | FormWidget::DisplayText(_)
            | FormWidget::ErrorText(_) => None,
        }
    }

    pub fn max_cursor(&self) -> usize {
        match self {
            FormWidget::InputBox { text, .. }
            | FormWidget::DisplayBox { text, .. }
            | FormWidget::SelectInput { text, .. } => text.len(),
            FormWidget::BooleanInput { value, .. } => value.to_string().len(),
            FormWidget::Button { .. }
            | FormWidget::Heading(_)
            | FormWidget::StaticText(_)
            | FormWidget::DisplayText(_)
            | FormWidget::ErrorText(_) => 0,
        }
    }

    pub fn to_value(&self) -> Option<String> {
        match self {
            FormWidget::InputBox { text, .. }
            | FormWidget::DisplayBox { text, .. }
            | FormWidget::SelectInput { text, .. } => Some(text.clone()),
            FormWidget::BooleanInput { value, .. } => Some(value.to_string()),
            FormWidget::Button { .. }
            | FormWidget::Heading(_)
            | FormWidget::StaticText(_)
            | FormWidget::DisplayText(_)
            | FormWidget::ErrorText(_) => None,
        }
    }

    pub fn height(&self, area: Rect) -> u16 {
        match self {
            FormWidget::InputBox { text, .. }
            | FormWidget::DisplayBox { text, .. }
            | FormWidget::SelectInput { text, .. } => {
                let lines = split_string(text, (area.width - 2) as usize);
                (2 + lines.len()) as u16
            }

            FormWidget::BooleanInput { value, .. } => {
                let value = value.to_string();
                let lines = split_string(&value, (area.width - 2) as usize);
                (2 + lines.len()) as u16
            }
            FormWidget::Button { .. } => 3,
            FormWidget::Heading(_) | FormWidget::StaticText(_) => 2,
            FormWidget::DisplayText(text) | FormWidget::ErrorText(text) => {
                if text.is_empty() {
                    0
                } else {
                    (text.len() as u16).div_ceil(area.width) + 1
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Form<
    T: IntoEnumIterator + ToString + FormItemIndex + TryInto<FormWidget, Error = E>,
    E: From<crate::error::RatatuiExtraError>,
> {
    pub cursor: usize,
    pub text_cursor: usize,
    pub form_focus: bool,
    pub items: Vec<FormWidget>,
    pub hide: HashMap<usize, bool>,
    pub everything_empty: bool,
    pub _phantom: PhantomData<T>,
}

impl<
        T: IntoEnumIterator + ToString + FormItemIndex + TryInto<FormWidget, Error = E>,
        E: From<crate::error::RatatuiExtraError>,
    > Form<T, E>
{
    pub fn init<F>(set_values_closure: F) -> Result<Self, E>
    where
        F: FnOnce(&mut Self) -> Result<(), E>,
    {
        let mut form = Self {
            cursor: 0,
            text_cursor: 0,
            form_focus: true,
            items: T::iter()
                .map(|item| item.try_into())
                .collect::<Result<Vec<FormWidget>, _>>()?,
            hide: HashMap::new(),
            everything_empty: false,
            _phantom: PhantomData,
        };
        for i in 0..form.items.len() {
            if form.is_valid_cursor(i) {
                break;
            } else {
                form.cursor += 1;
            }
        }
        set_values_closure(&mut form)?;
        form.text_cursor = form.items[form.cursor].max_cursor();

        Ok(form)
    }

    pub fn set_form_focus(&mut self, focus: bool) {
        self.form_focus = focus;
    }

    pub fn show_everything_empty(&mut self, empty: bool) {
        self.everything_empty = empty;
    }

    pub fn hide_item(&mut self, idx: T) {
        let index = idx.index();
        self.hide.insert(index, true);
        if self.cursor == index {
            self.advance_cursor();
        }
    }

    pub fn show_item(&mut self, idx: T) {
        self.hide.remove(&idx.index());
    }

    pub fn hidden_count(&self) -> usize {
        self.hide.len()
    }

    pub fn visible_count(&self) -> usize {
        self.items.len() - self.hidden_count()
    }

    pub fn valid_count(&self) -> usize {
        self.items
            .iter()
            .enumerate()
            .filter(|(i, _)| self.is_valid_cursor(*i))
            .count()
    }

    pub fn advance_cursor(&mut self) {
        if self.valid_count() > 0 {
            loop {
                self.cursor = (self.cursor + 1) % self.items.len();
                self.update_text_cursor();

                if self.is_valid_cursor(self.cursor) {
                    break;
                }
            }
        }
    }

    pub fn retreat_cursor(&mut self) {
        loop {
            self.cursor = (self.cursor + self.items.len() - 1) % self.items.len();
            self.update_text_cursor();

            if self.is_valid_cursor(self.cursor) {
                break;
            }
        }
    }

    pub fn update_text_cursor(&mut self) {
        self.text_cursor = self.items[self.cursor].max_cursor();
    }

    pub fn is_valid_cursor(&self, idx: usize) -> bool {
        if self.hide.contains_key(&idx) {
            return false;
        }

        match &self.items[idx] {
            FormWidget::Heading(_)
            | FormWidget::StaticText(_)
            | FormWidget::DisplayText(_)
            | FormWidget::ErrorText(_) => false,

            FormWidget::InputBox { .. }
            | FormWidget::DisplayBox { .. }
            | FormWidget::BooleanInput { .. }
            | FormWidget::SelectInput { .. }
            | FormWidget::Button { .. } => true,
        }
    }

    pub fn get_text(&self, idx: T) -> &String {
        match &self.items[idx.index()] {
            FormWidget::InputBox { text, .. } => text,
            FormWidget::DisplayBox { text, .. } => text,
            FormWidget::DisplayText(text) => text,
            FormWidget::ErrorText(text) => text,
            FormWidget::SelectInput { text, .. } => text,
            _ => unreachable!(),
        }
    }

    pub fn get_text_mut(&mut self, idx: T) -> &mut String {
        match &mut self.items[idx.index()] {
            FormWidget::InputBox { text, .. } => text,
            FormWidget::DisplayBox { text, .. } => text,
            FormWidget::DisplayText(text) => text,
            FormWidget::ErrorText(text) => text,
            FormWidget::SelectInput { text, .. } => text,
            _ => unreachable!(),
        }
    }

    pub fn get_boolean(&self, idx: T) -> bool {
        match &self.items[idx.index()] {
            FormWidget::BooleanInput { value, .. } => *value,
            _ => unreachable!(),
        }
    }

    pub fn get_boolean_mut(&mut self, idx: T) -> &mut bool {
        match &mut self.items[idx.index()] {
            FormWidget::BooleanInput { value, .. } => value,
            _ => unreachable!(),
        }
    }

    pub fn get_currency_mut(&mut self, idx: T) -> Option<&mut Option<String>> {
        match &mut self.items[idx.index()] {
            FormWidget::InputBox { currency, .. } => Some(currency),
            _ => None,
        }
    }

    pub fn get_popup_mut(&mut self, idx: T) -> &mut FilterSelectPopup<String> {
        match &mut self.items[idx.index()] {
            FormWidget::SelectInput { popup, .. } => popup,
            _ => unreachable!(),
        }
    }

    pub fn is_focused(&self, idx: T) -> bool {
        self.cursor == idx.index()
    }

    pub fn is_button_focused(&self) -> bool {
        matches!(self.items[self.cursor], FormWidget::Button { .. })
    }

    pub fn is_some_popup_open(&self) -> bool {
        self.items
            .iter()
            .any(|item| matches!(item, FormWidget::SelectInput { popup, .. } if popup.is_open()))
    }

    pub fn is_select_focused(&self) -> bool {
        matches!(self.items[self.cursor], FormWidget::SelectInput { .. })
    }

    pub fn current_label_enum(&self) -> Result<T, E> {
        T::iter().nth(self.cursor).ok_or_else(|| {
            crate::error::RatatuiExtraError::FormLabelNotAvailable {
                cursor: self.cursor,
                available: T::iter().map(|t| t.to_string()).collect(),
            }
            .into()
        })
    }

    pub fn handle_event<A, F1, F2>(
        &mut self,
        key_event: Option<&KeyEvent>,
        mut on_value_change: F2,
        mut on_button_press: F1,
    ) -> Result<A, E>
    where
        A: Act,
        F1: FnMut(T, &mut Self) -> Result<(), E>,
        F2: FnMut(T, &mut Self) -> Result<(), E>,
    {
        let mut result = A::default();

        if let Some(key_event) = key_event {
            if key_event.kind == KeyEventKind::Press {
                if !self.is_some_popup_open() {
                    match key_event.code {
                        KeyCode::Up => {
                            self.retreat_cursor();
                        }
                        KeyCode::Down | KeyCode::Tab => {
                            self.advance_cursor();
                        }
                        KeyCode::Enter => {
                            if !self.is_button_focused() && !self.is_select_focused() {
                                self.advance_cursor();
                            }
                        }

                        _ => {}
                    }
                }

                let value_before = self.items[self.cursor].to_value();

                match &mut self.items[self.cursor] {
                    FormWidget::InputBox { text, .. } => {
                        InputBox::handle_event(Some(key_event), text, &mut self.text_cursor);
                    }
                    FormWidget::DisplayBox { .. } => {
                        // we don't have to handle this as parent component will do it
                    }
                    FormWidget::BooleanInput { value, .. } => {
                        if matches!(
                            key_event.code,
                            KeyCode::Char(_) | KeyCode::Left | KeyCode::Right | KeyCode::Backspace
                        ) {
                            *value = !*value;
                            self.text_cursor = value.to_string().len();
                        }
                    }
                    FormWidget::SelectInput { text, popup, .. } => {
                        let is_open = popup.is_open();

                        let popup_result = popup.handle_event(Some(key_event), |selected| {
                            *text = selected.clone();
                            self.text_cursor = selected.len();
                            Ok(())
                        })?;
                        result.merge(popup_result);

                        if !is_open {
                            match key_event.code {
                                // Press any key to open the popup
                                KeyCode::Backspace | KeyCode::Char(_) | KeyCode::Enter => {
                                    popup.open();
                                }
                                _ => {}
                            }
                        }
                    }
                    FormWidget::Button { .. } => {
                        if matches!(key_event.code, KeyCode::Enter) {
                            on_button_press(self.current_label_enum()?, self)?
                        }
                    }
                    _ => {}
                }

                let value_after = self.items[self.cursor].to_value();
                if value_after != value_before {
                    on_value_change(self.current_label_enum()?, self)?;
                }
            }
        }
        Ok(result)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        let horizontal_layout = Layout::horizontal([Constraint::Min(3), Constraint::Length(1)]);
        let [left_area, scroll_area] = horizontal_layout.areas(area);

        // First check how much height the form will take
        let calc_virtual_form_height = |area: Rect| {
            self.items
                .iter()
                .enumerate()
                .filter(|(i, _)| !self.hide.contains_key(i))
                .fold(0, |acc, (_, w)| acc + w.height(area))
        };

        // By default we want to render form in the entire area
        let mut form_area = area;
        let mut virtual_form_height = calc_virtual_form_height(area);

        // But if form is overflowing we will render it in the left area only, and recalculate virtual_form_height
        let is_form_overflow = area.height < virtual_form_height;

        if is_form_overflow {
            form_area = left_area;
            virtual_form_height = calc_virtual_form_height(form_area);
        }

        let mut virtual_area = Rect::new(
            area.x,
            area.y,
            form_area.width,
            std::cmp::max(virtual_form_height, area.height),
        );
        let mut virtual_buf = Buffer::empty(virtual_area);

        let mut scroll_y: u16 = 0;
        let mut focused_item_height: u16 = 0;

        for (i, item) in self.items.iter().enumerate() {
            // Skip hidden items.
            if self.hide.contains_key(&i) {
                continue;
            }

            // This will hit only once and we record the scroll position of the focused item.
            if self.form_focus && self.cursor == i {
                scroll_y = virtual_area.y;
                focused_item_height = item.height(virtual_area);
            }

            // Render all form items in our virtual buffer.
            match item {
                FormWidget::Heading(heading) => {
                    heading.bold().render(virtual_area, &mut virtual_buf);
                    virtual_area.consume_height(2);
                }
                FormWidget::StaticText(text) => {
                    text.render(virtual_area, &mut virtual_buf);
                    virtual_area.consume_height(2);
                }
                FormWidget::InputBox {
                    label,
                    text,
                    empty_text,
                    currency,
                } => {
                    let widget = InputBox {
                        focus: self.form_focus && self.cursor == i,
                        label,
                        text: if !self.everything_empty {
                            text
                        } else {
                            &"".to_string()
                        },
                        empty_text: if !self.everything_empty {
                            *empty_text
                        } else {
                            Some("")
                        },
                        currency: currency.as_ref(),
                    };
                    let height_used = widget.height_used(virtual_area);

                    widget.render(virtual_area, &mut virtual_buf, &self.text_cursor, theme);
                    virtual_area.consume_height(height_used);
                }
                FormWidget::DisplayBox {
                    label,
                    text,
                    empty_text,
                } => {
                    let widget = InputBox {
                        focus: self.form_focus && self.cursor == i,
                        label,
                        text: if !self.everything_empty {
                            text
                        } else {
                            &"".to_string()
                        },
                        empty_text: if !self.everything_empty {
                            *empty_text
                        } else {
                            Some("")
                        },
                        currency: None,
                    };
                    let height_used = widget.height_used(virtual_area);

                    widget.render(virtual_area, &mut virtual_buf, &self.text_cursor, theme);
                    virtual_area.consume_height(height_used);
                }
                FormWidget::BooleanInput { label, value } => {
                    let widget = InputBox {
                        focus: self.form_focus && self.cursor == i,
                        label,
                        text: if !self.everything_empty {
                            &value.to_string()
                        } else {
                            &"".to_string()
                        },
                        empty_text: None,
                        currency: None,
                    };
                    let height_used = widget.height_used(virtual_area); // to see height based on width

                    widget.render(virtual_area, &mut virtual_buf, &self.text_cursor, theme);
                    virtual_area.consume_height(height_used);
                }
                FormWidget::SelectInput {
                    label,
                    text,
                    empty_text,
                    ..
                } => {
                    let widget = InputBox {
                        focus: self.form_focus && self.cursor == i,
                        label,
                        text,
                        empty_text: *empty_text,
                        currency: None,
                    };
                    let height_used = widget.height_used(virtual_area); // to see height based on width

                    widget.render(virtual_area, &mut virtual_buf, &self.text_cursor, theme);
                    virtual_area.consume_height(height_used);
                }
                FormWidget::Button { label } => {
                    Button {
                        focus: self.form_focus && self.cursor == i,
                        label,
                    }
                    .render(virtual_area, &mut virtual_buf, theme);

                    virtual_area.consume_height(3);
                }
                FormWidget::DisplayText(text) | FormWidget::ErrorText(text) => {
                    if !text.is_empty() {
                        Paragraph::new(Text::raw(text))
                            .wrap(Wrap { trim: false })
                            .render(virtual_area, &mut virtual_buf);
                        virtual_area
                            .consume_height((text.len() as u16).div_ceil(virtual_area.width));
                        virtual_area.consume_height(1);
                    }
                }
            }
        }

        // Ensure correctness, if this fails then there is inconsistency between FormWidget::height vs the above rendering code.
        if is_form_overflow {
            assert_eq!(virtual_area.height, 0);
        } else {
            assert_eq!(virtual_area.height, area.height - virtual_form_height);
        }

        if is_form_overflow {
            CustomScrollBar {
                cursor: scroll_y as usize,
                total_items: virtual_form_height as usize,
                paginate: true,
            }
            .render(scroll_area, buf);
        }

        let mut virtual_canvas_area = form_area;
        if is_form_overflow && scroll_y > form_area.height {
            virtual_canvas_area.y = scroll_y + focused_item_height - form_area.height;
        }

        // Only show contents that are visible, copy contents from virtual buffer to the actual buffer
        for (src_row, dst_row) in virtual_canvas_area.rows().zip(form_area.rows()) {
            for (src_col, dst_col) in src_row.columns().zip(dst_row.columns()) {
                if let Some(dst) = buf.cell_mut((dst_col.x, dst_col.y)) {
                    if let Some(src) = virtual_buf.cell((src_col.x, src_col.y)) {
                        *dst = src.clone();
                    }
                };
            }
        }
        // Render popups at the end so they appear on the top
        for item in &self.items {
            #[allow(clippy::single_match)]
            match item {
                FormWidget::SelectInput { popup, .. } => {
                    popup.render(area, buf, theme);
                }
                _ => {}
            }
        }
    }
}
