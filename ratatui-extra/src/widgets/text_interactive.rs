use gm_utils::{
    text_segment::{segmented_wrap, TokenKind, WrappedSegment},
    text_wrap::{has_new_line_char, text_wrap},
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, MouseButton, MouseEventKind},
    layout::{Constraint, Layout, Position, Rect},
    style::{Modifier, Style, Styled},
    widgets::Widget,
};
use std::borrow::Cow;

use crate::{
    act::Act,
    extensions::{MouseEventExt, PositionExt, RectExt, ThemedWidget},
    thematize::Thematize,
};

use super::scroll_bar::CustomScrollBar;

type VisibleText<'a> = (Vec<(Cow<'a, str>, bool)>, usize, Vec<WrappedSegment>);

#[derive(Default, Debug)]
pub struct TextInteractive {
    text: String,
    scroll_offset: usize,
    segment_idx: Option<usize>,
    mouse_drag_start: Option<Position>,
    mouse_drag_current: Option<Position>,
}

impl TextInteractive {
    pub fn with_text(mut self, text: String) -> Self {
        self.set_text(text, false);
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn into_text(self) -> String {
        self.text
    }

    pub fn set_text(&mut self, text: String, scroll_to_top: bool) {
        self.text = text;
        if scroll_to_top {
            self.scroll_offset = 0;
            self.segment_idx = None;
        }
    }

    pub fn is_focused(&self) -> bool {
        self.segment_idx.is_some()
    }

    pub fn lines_count(&self, width: usize) -> usize {
        text_wrap(&self.text, width as u16).len()
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self, width: usize, height: usize) {
        let lines = self.lines_count(width);
        if self.scroll_offset + height < lines {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_to_bottom(&mut self, width: usize, height: usize) {
        let lines = self.lines_count(width);
        if lines > height {
            self.scroll_offset = lines - height;
        } else {
            self.scroll_offset = 0;
        }
    }

    fn get_visible_text(&self, area: Rect) -> VisibleText<'_> {
        let (lines, segments) = segmented_wrap(&self.text, area.width);
        let new_line_chars = has_new_line_char(&self.text, &lines);
        let lines_len = lines.len();

        let visible_segments: Vec<WrappedSegment> = segments
            .into_iter()
            .filter(|seg| {
                (seg.start_line >= self.scroll_offset
                    && seg.start_line < self.scroll_offset + area.height as usize)
                    || (seg.end_line >= self.scroll_offset
                        && seg.end_line < self.scroll_offset + area.height as usize)
                    || (seg.start_line < self.scroll_offset
                        && seg.end_line >= self.scroll_offset + area.height as usize)
            })
            .map(|mut seg| {
                if seg.start_line < self.scroll_offset {
                    seg.start_line = self.scroll_offset;
                    seg.start_char_idx = 0;
                }

                if seg.end_line >= self.scroll_offset + area.height as usize {
                    seg.end_line = self.scroll_offset + area.height as usize - 1;
                    seg.end_char_idx = lines[seg.end_line].len();
                }

                seg
            })
            .collect();

        let visible_lines = lines
            .into_iter()
            .skip(self.scroll_offset)
            .take(area.height as usize)
            .zip(
                new_line_chars
                    .into_iter()
                    .skip(self.scroll_offset)
                    .take(area.height as usize),
            )
            .collect();

        // TODO we also need to calculate the line_idx beyond visible_segments where there's a valid segment
        // This is for the purpose when user press TAB when there is nothing in visible segments but there is
        // something beyond so we want to scroll there.

        (visible_lines, lines_len, visible_segments)
    }

    // TODO currently handle_event and render calls get_visible_text everytime. It might be ok for now but
    // ideally we should cache the result and update it when text changes or resize occurs.
    // fn update_visible_text(&mut self, area: Rect) -> VisibleData {
    //     let (lines, total_lines, segments) = self.get_visible_text(area);
    //     VisibleData {
    //         lines: lines
    //             .into_iter()
    //             .map(Cow::into_owned)
    //             .collect::<Vec<String>>(),
    //         total_lines,
    //         segments,
    //     }
    // }

    pub fn handle_event(&mut self, event: Option<&Event>, area: Rect, actions: &mut impl Act) {
        let text_area = area.margin_right(1);

        if let Some(event) = event {
            match event {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Up => {
                            self.scroll_up();
                        }
                        KeyCode::Down => {
                            self.scroll_down(area.width as usize, area.height as usize);
                        }
                        KeyCode::Tab => {
                            let (_, _, segments) = self.get_visible_text(area);
                            if !segments.is_empty() {
                                if let Some(segment_idx) = &mut self.segment_idx {
                                    if let Some((found_idx, _)) = segments
                                        .iter()
                                        .enumerate()
                                        .find(|(_, segment)| segment.idx == *segment_idx)
                                    {
                                        if let Some(next_segment) = segments.get(found_idx + 1) {
                                            *segment_idx = next_segment.idx;
                                        } else {
                                            *segment_idx = segments[0].idx;
                                            // TODO scroll down upto the next segment
                                        }
                                    } else {
                                        *segment_idx = segments[0].idx;
                                    }
                                } else {
                                    self.segment_idx = Some(segments[0].idx);
                                }
                            }
                        }
                        KeyCode::Esc => {
                            self.segment_idx = None;
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if text_area.contains(mouse_event.position()) {
                            let (_, _, segments) = self.get_visible_text(area);

                            Self::segments_iter(
                                segments,
                                text_area,
                                self.scroll_offset,
                                |segment, span_area| {
                                    if span_area.contains(mouse_event.position()) {
                                        match &segment.kind {
                                            TokenKind::Url(url) => {
                                                actions.open_url(
                                                    url.clone(),
                                                    Some(mouse_event.position()),
                                                );
                                            }
                                            TokenKind::Hex(str) => {
                                                actions.copy_to_clipboard(
                                                    str.clone(),
                                                    Some(mouse_event.position()),
                                                );
                                            }
                                        }
                                    }
                                },
                            );

                            if self.mouse_drag_start.is_none() {
                                self.mouse_drag_start =
                                    Some(mouse_event.position().nearest_inner(text_area));
                            }
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        self.mouse_drag_current =
                            Some(mouse_event.position().nearest_inner(text_area));
                    }
                    MouseEventKind::Up(MouseButton::Left) => {
                        if text_area.contains(mouse_event.position()) {
                            if let Some(start) = self.mouse_drag_start.take() {
                                let current = mouse_event.position();
                                if let Some(string) =
                                    self.get_string_by_positions(start, current, text_area)
                                {
                                    actions.copy_to_clipboard(string, Some(mouse_event.position()));
                                }
                                self.mouse_drag_current = None;
                            }
                        }
                    }
                    MouseEventKind::Moved => {
                        let (_, _, segments) = self.get_visible_text(area);

                        let mut hovered_on_some_segment = false;
                        Self::segments_iter(
                            segments,
                            text_area,
                            self.scroll_offset,
                            |segment, span_area| {
                                if span_area.contains(mouse_event.position()) {
                                    hovered_on_some_segment = true;
                                    self.segment_idx = Some(segment.idx);
                                }
                            },
                        );

                        if !hovered_on_some_segment {
                            self.segment_idx = None;
                        }

                        self.mouse_drag_start = None;
                    }
                    MouseEventKind::ScrollUp => {
                        self.scroll_up();
                    }
                    MouseEventKind::ScrollDown => {
                        self.scroll_down(area.width as usize, area.height as usize);
                    }
                    _ => {}
                },
                // TODO implement resize handling so that we don't frequently call get_visible_text
                // Event::Resize(_, _) => {}
                _ => {}
            }
        }

        if self.segment_idx.is_some() {
            actions.ignore_esc();
        }
    }

    fn segments_iter<F>(
        segments: Vec<WrappedSegment>,
        text_area: Rect,
        scroll_offset: usize,
        mut callback: F,
    ) where
        F: FnMut(&WrappedSegment, Rect),
    {
        for segment in segments {
            for line_idx in segment.start_line..=segment.end_line {
                let line_area = Rect {
                    x: text_area.x,
                    y: text_area.y + (line_idx.saturating_sub(scroll_offset)) as u16,
                    width: text_area.width,
                    height: 1,
                };

                let Some((start, end)) =
                    (if line_idx == segment.start_line && line_idx == segment.end_line {
                        Some((segment.start_char_idx, segment.end_char_idx))
                    } else if line_idx == segment.start_line {
                        Some((segment.start_char_idx, line_area.width as usize))
                    } else if line_idx == segment.end_line {
                        Some((0, segment.end_char_idx))
                    } else if line_idx > segment.start_line && line_idx < segment.end_line {
                        Some((0, line_area.width as usize))
                    } else {
                        None
                    })
                else {
                    continue;
                };

                let span_area = line_area
                    .margin_left(start as u16)
                    // TODO the text wrap has a bug, sometimes it seems that lines are
                    // longer than the constrained width.
                    .margin_right(text_area.width.saturating_sub(end as u16));

                callback(&segment, span_area);
            }
        }
    }

    /// Get the string between the two positions if any. This should not result in an error
    /// unless there is a bug in the code.
    fn get_string_by_positions(
        &self,
        start_position: Position,
        current_position: Position,
        text_area: Rect,
    ) -> Option<String> {
        let (start_position, end_position) = start_position.sort(current_position);

        let (lines, _, _) = self.get_visible_text(text_area);
        if lines.is_empty() {
            return None;
        }

        let calc_idx = |position: Position| -> (usize, usize) {
            // ensured: that position is within text_area
            let mut line_idx = (position.y - text_area.y) as usize;
            let mut char_idx = (position.x - text_area.x) as usize;

            if line_idx >= lines.len() {
                // position is beyond the last line (but within text_area)
                line_idx = lines.len() - 1; // ensured: lines.len() > 0
                char_idx = lines[line_idx].0.len().saturating_sub(1);
            } else {
                // position is within the visible lines
                char_idx = char_idx.min(lines[line_idx].0.len().saturating_sub(1));
            }

            (line_idx, char_idx)
        };

        let (start_line, start_idx) = calc_idx(start_position);
        let (end_line, end_idx) = calc_idx(end_position);

        // ensured: start_line <= end_line
        let mut string = String::with_capacity(
            // rough capacity estimate
            (end_line - start_line + 1) * (text_area.width as usize),
        );

        if start_line == end_line {
            // selection is within the same line
            if start_idx == end_idx {
                // nothing is selected
                return None;
            }

            let (line, _) = &lines[start_line]; // ensured: start_line < lines.len()
            string.push_str(&line[start_idx..=end_idx]); // ensured: idx < lines[idx].len()
        } else {
            // first line of selection
            let (line, has_new_line_char) = &lines[start_line];
            string.push_str(&line[start_idx..]);
            if *has_new_line_char {
                string.push('\n');
            }

            // lines in between
            if end_line - start_line > 1 {
                for (line, has_new_line_char) in lines
                    .iter()
                    .skip(start_line + 1)
                    .take(end_line - start_line - 1)
                {
                    string.push_str(line);
                    if *has_new_line_char {
                        string.push('\n');
                    }
                }
            }

            // last line of selection
            let (line, _) = &lines[end_line];
            string.push_str(&line[..=end_idx]);
        }

        Some(string)
    }
}

impl ThemedWidget for TextInteractive {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        let [text_area, scroll_area] =
            Layout::horizontal([Constraint::Min(1), Constraint::Length(1)]).areas(area);
        let (lines, total, segments) = self.get_visible_text(text_area);

        let text_area = if total > area.height as usize {
            CustomScrollBar {
                cursor: self.scroll_offset,
                total_items: total - area.height as usize + 1,
                paginate: false,
            }
            .render(scroll_area, buf, theme);
            text_area
        } else {
            area
        };

        for (i, (line, _)) in lines.iter().enumerate() {
            let line_area = text_area.margin_top(i as u16);
            line.set_style(theme.style_dim()).render(line_area, buf);
        }

        Self::segments_iter(
            segments,
            text_area,
            self.scroll_offset,
            |segment, span_area| {
                let mut style = Style::default().add_modifier(Modifier::UNDERLINED);

                let is_segment_focused = self.segment_idx.is_some_and(|idx| idx == segment.idx);

                if is_segment_focused {
                    style = theme.select_focused();
                }

                buf.set_style(span_area, style);
            },
        );

        if let Some((start, current)) = self.mouse_drag_start.zip(self.mouse_drag_current) {
            let (start, end) = start.sort(current);

            if start.y == end.y {
                // We are selecting in the same line
                buf.set_style(
                    Rect {
                        x: start.x.min(end.x),
                        y: start.y.min(end.y),
                        width: start.x.abs_diff(end.x) + 1,
                        height: 1,
                    },
                    theme.select_focused(),
                )
            } else {
                // We are selecting across multiple lines, start with highlighting the first line
                buf.set_style(
                    Rect {
                        x: start.x,
                        y: start.y,
                        width: text_area.width + text_area.x - start.x,
                        height: 1,
                    },
                    theme.select_focused(),
                );

                // Highlight all the lines in between
                if end.y - start.y > 1 {
                    buf.set_style(
                        Rect {
                            x: text_area.x,
                            y: start.y + 1,
                            width: text_area.width,
                            height: end.y - start.y - 1,
                        },
                        theme.select_focused(),
                    );
                }

                // Highlight the last line
                buf.set_style(
                    Rect {
                        x: text_area.x,
                        y: end.y,
                        width: end.x - text_area.x + 1,
                        height: 1,
                    },
                    theme.select_focused(),
                );
            }
        }
    }
}
