use std::borrow::Cow;
use std::{collections::HashMap, marker::PhantomData};

use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::text::Span;
use ratatui::widgets::WidgetRef;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Wrap},
};
use strum::IntoEnumIterator;

use super::{button::Button, input_box::InputBox};
use crate::act::Act;
use crate::boolean_input::BooleanInput;
use crate::button::ButtonResult;
use crate::event::WidgetEvent;
use crate::extensions::{MouseEventExt, RectExt, RenderTextWrapped, WidgetHeight};
use crate::widgets::scroll_bar::CustomScrollBar;
use crate::{thematize::Thematize, widgets::filter_select_popup::FilterSelectPopup};

pub enum FormEvent<T> {
    ValueChanged(T),
    ButtonPressed(T),
}

enum CursorAction {
    Prev,
    Next,
    None,
}

pub trait FormItemIndex {
    fn index(self) -> usize;
}

/// Types of widgets that can be handled and rendered by Form.
#[derive(Debug)]
pub enum FormWidget {
    /// Displays texts with a stronger style
    Heading(&'static str),

    /// Displays text with a dimmer style
    StaticText(&'static str),

    /// Take user input as a string
    InputBox { widget: InputBox },

    /// Provide a switch to input a boolean
    BooleanInput { widget: BooleanInput },

    /// Display as a input immutable
    // TODO explore if this can be removed
    DisplayBox { widget: InputBox },

    /// Renders a popup to select an option from
    SelectInput {
        widget: InputBox,
        popup: FilterSelectPopup<String>,
    },

    /// Button to interact
    Button { widget: Button },

    /// Displays text with a dimmer style
    // TODO explore if this can be combined with StaticText
    DisplayText(String),

    /// Displays error text with a red style
    ErrorText(String),

    /// Simply renders a new line to keep space between two elements
    LineBreak,
}

impl FormWidget {
    pub fn label(&self) -> Option<&'static str> {
        match self {
            FormWidget::InputBox { widget }
            | FormWidget::DisplayBox { widget, .. }
            | FormWidget::SelectInput { widget, .. } => Some(widget.label),

            FormWidget::BooleanInput { widget, .. } => Some(widget.label),

            FormWidget::Button { widget, .. } => Some(widget.label),

            FormWidget::Heading(_)
            | FormWidget::StaticText(_)
            | FormWidget::DisplayText(_)
            | FormWidget::ErrorText(_)
            | FormWidget::LineBreak => None,
        }
    }

    pub fn to_value(&self) -> Option<String> {
        match self {
            FormWidget::InputBox { widget, .. }
            | FormWidget::DisplayBox { widget, .. }
            | FormWidget::SelectInput { widget, .. } => Some(widget.get_text().to_string()),
            FormWidget::BooleanInput { .. } => None,

            FormWidget::Button { .. }
            | FormWidget::Heading(_)
            | FormWidget::StaticText(_)
            | FormWidget::DisplayText(_)
            | FormWidget::ErrorText(_)
            | FormWidget::LineBreak => None,
        }
    }

    pub fn height(&self, area: Rect) -> u16 {
        match self {
            FormWidget::InputBox { widget, .. }
            | FormWidget::DisplayBox { widget, .. }
            | FormWidget::SelectInput { widget, .. } => widget.height_used(area),

            FormWidget::BooleanInput { .. } => 3,
            FormWidget::Button { .. } => 3,
            FormWidget::Heading(text) | FormWidget::StaticText(text) => {
                (text.len() as u16).div_ceil(area.width) + 1
            }
            FormWidget::DisplayText(text) | FormWidget::ErrorText(text) => {
                if text.is_empty() {
                    0
                } else {
                    (text.len() as u16).div_ceil(area.width) + 1
                }
            }
            FormWidget::LineBreak => 1,
        }
    }

    pub fn width(&self, area: Rect) -> u16 {
        match self {
            FormWidget::Button { widget, .. } => widget.area(area).width,
            _ => area.width,
        }
    }
}

#[derive(Debug)]
pub struct Form<
    T: IntoEnumIterator + ToString + FormItemIndex + TryInto<FormWidget, Error = E>,
    E: From<crate::error::RatatuiExtraError>,
> {
    cursor: usize,
    scroll_y: u16,
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
            scroll_y: 0,
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

                if self.is_valid_cursor(self.cursor) {
                    break;
                }
            }
        }
    }

    pub fn retreat_cursor(&mut self) {
        loop {
            self.cursor = (self.cursor + self.items.len() - 1) % self.items.len();
            // self.update_text_cursor();

            if self.is_valid_cursor(self.cursor) {
                break;
            }
        }
    }

    fn handle_cursor(&mut self, area: Rect, cursor_action: CursorAction) {
        let (form_area, _, _) = self.get_form_area(area);

        match cursor_action {
            CursorAction::Prev => {
                self.retreat_cursor();
            }
            CursorAction::Next => {
                self.advance_cursor();
            }
            CursorAction::None => {}
        };

        let next_cursor = self.cursor;

        let mut height_before_next_item = 0;
        let mut current_item_height = 0;
        for (i, item) in self.items.iter().enumerate() {
            if self.hide.contains_key(&i) {
                continue;
            }

            let height = item.height(form_area);

            if i < next_cursor {
                height_before_next_item += height;
            } else if i == next_cursor {
                current_item_height = height;
            }
        }

        if Some(next_cursor) == self.first_valid_cursor() {
            // scroll to top
            self.scroll_y = 0;
        } else if self.scroll_y <= height_before_next_item
            && self.scroll_y + form_area.height > height_before_next_item + current_item_height
        {
            // do nothing, item is already in view
        } else if self.scroll_y > height_before_next_item {
            // scroll up to show the item at the top
            self.scroll_y = height_before_next_item;
        } else if self.scroll_y + form_area.height <= height_before_next_item + current_item_height
        {
            // scroll down to show the item at the bottom
            if height_before_next_item + current_item_height > form_area.height {
                self.scroll_y = height_before_next_item + current_item_height - form_area.height;
            } else {
                self.scroll_y = 0;
            }
        }
    }

    pub fn is_valid_cursor(&self, idx: usize) -> bool {
        if self.hide.contains_key(&idx) {
            return false;
        }

        match &self.items[idx] {
            FormWidget::Heading(_)
            | FormWidget::StaticText(_)
            | FormWidget::DisplayText(_)
            | FormWidget::ErrorText(_)
            | FormWidget::LineBreak => false,

            FormWidget::InputBox { .. }
            | FormWidget::DisplayBox { .. }
            | FormWidget::BooleanInput { .. }
            | FormWidget::SelectInput { .. }
            | FormWidget::Button { .. } => true,
        }
    }

    pub fn get_text(&self, idx: T) -> Cow<'_, str> {
        match &self.items[idx.index()] {
            FormWidget::InputBox { widget, .. } | FormWidget::DisplayBox { widget, .. } => {
                Cow::Borrowed(widget.get_text())
            }
            FormWidget::SelectInput { popup, .. } => Cow::Owned(popup.display_selection()),

            FormWidget::DisplayText(text) => Cow::Borrowed(text),
            FormWidget::ErrorText(text) => Cow::Borrowed(text),
            _ => unreachable!(),
        }
    }

    pub fn set_text(&mut self, idx: T, text: String) {
        match &mut self.items[idx.index()] {
            FormWidget::InputBox { widget, .. }
            | FormWidget::DisplayBox { widget, .. }
            | FormWidget::SelectInput { widget, .. } => widget.set_text(text),
            FormWidget::DisplayText(current) | FormWidget::ErrorText(current) => {
                *current = text;
            }
            _ => unreachable!(),
        }
    }

    pub fn get_boolean(&self, idx: T) -> bool {
        match &self.items[idx.index()] {
            FormWidget::BooleanInput { widget, .. } => widget.value,
            _ => unreachable!(),
        }
    }

    pub fn get_boolean_mut(&mut self, idx: T) -> &mut bool {
        match &mut self.items[idx.index()] {
            FormWidget::BooleanInput { widget, .. } => &mut widget.value,
            _ => unreachable!(),
        }
    }

    pub fn get_currency_mut(&mut self, idx: T) -> Option<&mut Option<String>> {
        match &mut self.items[idx.index()] {
            FormWidget::InputBox { widget, .. } => Some(&mut widget.currency),
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

    pub fn is_button_idx(&self, idx: usize) -> bool {
        matches!(self.items[idx], FormWidget::Button { .. })
    }

    pub fn get_button_hover_mut_idx(&mut self, idx: usize) -> &mut bool {
        match &mut self.items[idx] {
            FormWidget::Button { widget } => &mut widget.hover_focus,
            _ => unreachable!(),
        }
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

    pub fn current_label_enum(&self) -> crate::Result<T> {
        T::iter()
            .nth(self.cursor)
            .ok_or_else(|| crate::Error::FormLabelNotAvailable {
                cursor: self.cursor,
                available: T::iter().map(|t| t.to_string()).collect(),
            })
    }

    pub fn handle_event<A>(
        &mut self,
        event: Option<&WidgetEvent>,
        area: Rect,
        popup_area: Rect,
        actions: &mut A,
    ) -> crate::Result<Option<FormEvent<T>>>
    where
        A: Act,
    {
        let mut result = None;

        if let Some(event) = event {
            match event {
                WidgetEvent::InputEvent(Event::Key(key_event)) => {
                    if key_event.kind == KeyEventKind::Press && !self.is_some_popup_open() {
                        match key_event.code {
                            KeyCode::Up => {
                                self.handle_cursor(area, CursorAction::Prev);
                            }
                            KeyCode::Down | KeyCode::Tab => {
                                self.handle_cursor(area, CursorAction::Next);
                            }
                            KeyCode::Enter => {
                                if !self.is_button_focused() && !self.is_select_focused() {
                                    self.handle_cursor(area, CursorAction::Next);
                                }
                            }

                            _ => {}
                        }
                    }
                }
                WidgetEvent::InputEvent(Event::Mouse(mouse_event)) => {
                    if area.contains(mouse_event.position()) {
                        if mouse_event.is_left_click() {
                            if let Some(i) = self.get_clicked_item(area, mouse_event.position()) {
                                if self.is_valid_cursor(i) {
                                    self.cursor = i;
                                    self.handle_cursor(area, CursorAction::None);
                                }
                            }
                        } else if mouse_event.is(MouseEventKind::ScrollUp) {
                            if self.scroll_y > 0 {
                                self.scroll_y = self.scroll_y.saturating_sub(1);
                            }
                        } else if mouse_event.is(MouseEventKind::ScrollDown) {
                            let (form_area, _, virtual_form_height) = self.get_form_area(area);
                            if self.scroll_y + form_area.height < virtual_form_height {
                                self.scroll_y = self.scroll_y.saturating_add(1);
                            }
                        } else if mouse_event.is(MouseEventKind::Moved) {
                            let item_idx = self.get_clicked_item(area, mouse_event.position());
                            if item_idx.is_some_and(|idx| self.is_button_idx(idx)) {
                                // Highlight the button if mouse hovers it
                                *self.get_button_hover_mut_idx(item_idx.unwrap()) = true;
                            } else {
                                // Remove highlight from all the buttons in this form
                                for i in 0..self.items.len() {
                                    if self.is_button_idx(i) {
                                        *self.get_button_hover_mut_idx(i) = false;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            let value_before = self.items[self.cursor].to_value();

            let item_area = self.get_area_for_item(area, self.cursor);
            match &mut self.items[self.cursor] {
                FormWidget::InputBox { widget, .. } => {
                    widget.handle_event(Some(event), item_area, actions);
                }
                FormWidget::DisplayBox { .. } => {
                    // we don't have to handle this as parent component will do it
                }
                FormWidget::BooleanInput { widget, .. } => {
                    widget.handle_event(event.input_event(), item_area, actions);
                }
                FormWidget::SelectInput { widget, popup } => {
                    let is_open = popup.is_open();

                    widget.handle_event(Some(event), area, actions);

                    if let Some(selection) =
                        popup.handle_event(event.input_event(), popup_area, actions)
                    {
                        widget.set_text(selection.to_string());
                    }

                    if !is_open {
                        if let Some(key_event) = event.key_event() {
                            match key_event.code {
                                // Press any key to open the popup
                                KeyCode::Backspace | KeyCode::Char(_) | KeyCode::Enter => {
                                    popup.open();
                                }
                                _ => {}
                            }
                        }
                    }
                }
                FormWidget::Button { widget } => {
                    if let Some(ButtonResult::Pressed) =
                        widget.handle_event(event.input_event(), item_area, self.form_focus)
                    {
                        result = Some(FormEvent::ButtonPressed(self.current_label_enum()?));
                    }
                }
                _ => {}
            }

            let value_after = self.items[self.cursor].to_value();
            if value_after != value_before {
                result = Some(FormEvent::ValueChanged(self.current_label_enum()?));
            }
        }

        Ok(result)
    }

    fn calc_virtual_form_height(&self, area: Rect) -> u16 {
        self.items
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.hide.contains_key(i))
            .fold(0, |acc, (_, w)| acc + w.height(area))
    }

    // TODO combine with areas function and return a struct
    fn get_form_area(&self, area: Rect) -> (Rect, bool, u16) {
        let virtual_form_height = self.calc_virtual_form_height(area);
        let is_form_overflow = area.height < virtual_form_height;
        if is_form_overflow {
            let [left_area, _] = Self::areas(area);
            (left_area, is_form_overflow, virtual_form_height)
        } else {
            (area, is_form_overflow, virtual_form_height)
        }
    }

    fn first_valid_cursor(&self) -> Option<usize> {
        (0..self.items.len()).find(|&i| self.is_valid_cursor(i))
    }

    fn get_clicked_item(&self, area: Rect, position: Position) -> Option<usize> {
        let (form_area, _, _) = self.get_form_area(area);

        let clicked_x = position.x.saturating_sub(form_area.x);
        let clicked_y = self.scroll_y + position.y.saturating_sub(form_area.y);

        let mut scroll_y = 0;
        for (i, item) in self.items.iter().enumerate() {
            // Skip hidden items.
            if self.hide.contains_key(&i) {
                continue;
            }

            let item_width = item.width(area);
            let item_height = item.height(area);
            if clicked_y >= scroll_y && clicked_y < scroll_y + item_height && clicked_x < item_width
            {
                return Some(i);
            }
            scroll_y += item_height;
        }

        None
    }

    fn get_area_for_item(&self, area: Rect, idx: usize) -> Rect {
        let (form_area, _, _) = self.get_form_area(area);

        let mut item_virtual_y = 0;

        let mut item_height = None;

        for (i, item) in self.items.iter().enumerate() {
            if self.hide.contains_key(&i) {
                continue;
            }

            let h = item.height(form_area);

            if i == idx {
                item_height = Some(h);
                break;
            }
            item_virtual_y += h;
        }

        let item_height = item_height.expect("item_height should be Some");

        let mut item_area = form_area
            .height_consumed(item_virtual_y - self.scroll_y)
            .expect("not able to consume height");

        item_area.height = item_height;
        item_area
    }

    fn areas(area: Rect) -> [Rect; 2] {
        let [form_area, scroll_area] =
            Layout::horizontal([Constraint::Min(3), Constraint::Length(1)]).areas(area);
        [form_area, scroll_area]
    }

    pub fn render(&self, area: Rect, popup_area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        // In case there are few form elements, we don't need a scroll bar, in that case area == form_area.
        // However if form overflows the visible area, we need to reserve scroll bar area.
        let (form_area, is_form_overflow, virtual_form_height) = self.get_form_area(area);

        let mut virtual_area = Rect::new(
            area.x,
            area.y,
            form_area.width,
            std::cmp::max(virtual_form_height, area.height),
        );
        let mut virtual_buf = Buffer::empty(virtual_area);

        for (i, item) in self.items.iter().enumerate() {
            // Skip hidden items.
            if self.hide.contains_key(&i) {
                continue;
            }

            // Render all form items in our virtual buffer.
            match item {
                FormWidget::Heading(heading) => {
                    Span::raw(*heading)
                        .style(theme.style())
                        .render_wrapped(virtual_area, &mut virtual_buf);
                    virtual_area.consume_height(item.height(virtual_area));
                }
                FormWidget::StaticText(text) => {
                    Span::raw(*text)
                        .style(theme.style_dim())
                        .render_wrapped(virtual_area, &mut virtual_buf);
                    virtual_area.consume_height(item.height(virtual_area));
                }
                FormWidget::InputBox { widget, .. } => {
                    let height_used = item.height(virtual_area);

                    widget.render(
                        virtual_area,
                        &mut virtual_buf,
                        self.form_focus && self.cursor == i,
                        theme,
                    );
                    virtual_area.consume_height(height_used);
                }
                FormWidget::DisplayBox { widget, .. } => {
                    let height_used = item.height(virtual_area);

                    widget.render(
                        virtual_area,
                        &mut virtual_buf,
                        self.form_focus && self.cursor == i,
                        theme,
                    );
                    virtual_area.consume_height(height_used);
                }
                FormWidget::BooleanInput { widget, .. } => {
                    let height_used = item.height(virtual_area);

                    widget.render(
                        virtual_area,
                        &mut virtual_buf,
                        self.form_focus && self.cursor == i,
                        theme,
                    );
                    virtual_area.consume_height(height_used);
                }
                FormWidget::SelectInput { widget, .. } => {
                    let height_used = item.height(virtual_area);

                    widget.render(
                        virtual_area,
                        &mut virtual_buf,
                        self.form_focus && self.cursor == i,
                        theme,
                    );
                    virtual_area.consume_height(height_used);
                }
                FormWidget::Button { widget } => {
                    widget.render(
                        virtual_area,
                        &mut virtual_buf,
                        self.form_focus && self.cursor == i,
                        theme,
                    );

                    virtual_area.consume_height(item.height(virtual_area));
                }
                FormWidget::DisplayText(text) | FormWidget::ErrorText(text) => {
                    if !text.is_empty() {
                        Paragraph::new(Text::raw(text))
                            .wrap(Wrap { trim: false })
                            .render_ref(virtual_area, &mut virtual_buf);

                        virtual_area.consume_height(item.height(virtual_area));
                    }
                }
                FormWidget::LineBreak => {
                    // Just consume 1 height
                    virtual_area.consume_height(item.height(virtual_area));
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
            let [_, scroll_area] = Self::areas(area);
            CustomScrollBar {
                // Due to resize, self.scroll_y might be out of bounds, so we clamp it.
                cursor: self.scroll_y.min(virtual_form_height - area.height) as usize,
                total_items: (virtual_form_height - area.height + 1) as usize,
                paginate: false,
            }
            .render(scroll_area, buf, theme);
        }

        let mut virtual_canvas_area = form_area;
        if is_form_overflow {
            virtual_canvas_area.y = area.y + self.scroll_y;
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
                    popup.render(popup_area, buf, theme);
                }
                _ => {}
            }
        }
    }
}
