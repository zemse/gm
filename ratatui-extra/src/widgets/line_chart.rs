use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind, MouseEvent, MouseEventKind},
    layout::{Position, Rect},
    style::Color,
    widgets::{
        canvas::{Canvas, Line},
        Widget,
    },
};

use crate::extensions::{MouseEventExt, ThemedWidget};
use crate::thematize::Thematize;

pub enum LineChartEvent {
    Panned { dx: f64, dy: f64 },
    Zoomed { factor: f64 },
    Clicked { x: f64, y: f64 },
    Hover { x: f64, y: f64 },
}

#[derive(Debug)]
pub struct LineChart {
    data: Vec<(f64, f64)>,
    x_view: [f64; 2],
    y_view: [f64; 2],
    is_dragging: bool,
    last_mouse_x: u16,
    last_mouse_y: u16,
}

impl Default for LineChart {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            x_view: [0.0, 20.0],
            y_view: [-20.0, 20.0],
            is_dragging: false,
            last_mouse_x: 0,
            last_mouse_y: 0,
        }
    }
}

impl LineChart {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bounds(mut self, x: [f64; 2], y: [f64; 2]) -> Self {
        self.x_view = x;
        self.y_view = y;
        self
    }

    pub fn set_points(&mut self, pts: Vec<(f64, f64)>) {
        self.data = pts;
    }

    #[inline]
    fn pan_by(&mut self, dx_px: i16, dy_px: i16, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let x_span = self.x_view[1] - self.x_view[0];
        let y_span = self.y_view[1] - self.y_view[0];
        let dx = (dx_px as f64 / area.width as f64) * x_span;
        let dy = -(dy_px as f64 / area.height as f64) * y_span; // invert y
        self.x_view[0] += dx;
        self.x_view[1] += dx;
        self.y_view[0] += dy;
        self.y_view[1] += dy;
    }

    fn zoom_at(&mut self, factor: f64, mouse: Option<(u16, u16)>, area: Rect) {
        let (mx, my) = mouse.unwrap_or((area.x + area.width / 2, area.y + area.height / 2));
        let mxn = if area.width > 0 {
            (mx - area.x) as f64 / area.width as f64
        } else {
            0.5
        };
        let myn = if area.height > 0 {
            (my - area.y) as f64 / area.height as f64
        } else {
            0.5
        };

        let (x0, x1) = (self.x_view[0], self.x_view[1]);
        let xmid = x0 + (x1 - x0) * mxn;
        let xhalf = (x1 - x0) * factor * 0.5;
        self.x_view = [xmid - xhalf, xmid + xhalf];

        let (y0, y1) = (self.y_view[0], self.y_view[1]);
        let ymid = y0 + (y1 - y0) * (1.0 - myn);
        let yhalf = (y1 - y0) * factor * 0.5;
        self.y_view = [ymid - yhalf, ymid + yhalf];
    }

    pub fn handle_event(
        &mut self,
        ev: Option<&Event>,
        area: Rect,
    ) -> crate::Result<Option<LineChartEvent>> {
        let mut out = None;

        if let Some(e) = ev {
            match e {
                Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
                    KeyCode::Left => {
                        self.pan_by(-3, 0, area);
                        out = Some(LineChartEvent::Panned { dx: -3.0, dy: 0.0 });
                    }
                    KeyCode::Right => {
                        self.pan_by(3, 0, area);
                        out = Some(LineChartEvent::Panned { dx: 3.0, dy: 0.0 });
                    }
                    KeyCode::Up => {
                        self.pan_by(0, -1, area);
                        out = Some(LineChartEvent::Panned { dx: 0.0, dy: -1.0 });
                    }
                    KeyCode::Down => {
                        self.pan_by(0, 1, area);
                        out = Some(LineChartEvent::Panned { dx: 0.0, dy: 1.0 });
                    }
                    KeyCode::Char('+') => {
                        self.zoom_at(0.9, None, area);
                        out = Some(LineChartEvent::Zoomed { factor: 0.9 });
                    }
                    KeyCode::Char('-') => {
                        self.zoom_at(1.1, None, area);
                        out = Some(LineChartEvent::Zoomed { factor: 1.1 });
                    }
                    _ => {}
                },
                Event::Mouse(m) => {
                    out = self.handle_mouse(*m, area)?;
                }
                _ => {}
            }
        }
        Ok(out)
    }

    fn handle_mouse(&mut self, m: MouseEvent, area: Rect) -> crate::Result<Option<LineChartEvent>> {
        let mut out = None;
        if area.contains(m.position()) {
            match m.kind {
                MouseEventKind::Down(_) if m.is_left_click() => {
                    self.is_dragging = true;
                    self.last_mouse_x = m.column;
                    self.last_mouse_y = m.row;
                    if let Some((x, y)) = self.term_to_data(m.column, m.row, area) {
                        out = Some(LineChartEvent::Clicked { x, y });
                    }
                }
                MouseEventKind::Up(_) => self.is_dragging = false,
                MouseEventKind::Drag(_) if self.is_dragging => {
                    let dx = m.column as i16 - self.last_mouse_x as i16;
                    let dy = m.row as i16 - self.last_mouse_y as i16;
                    self.pan_by(-dx, -dy, area);
                    self.last_mouse_x = m.column;
                    self.last_mouse_y = m.row;
                    out = Some(LineChartEvent::Panned {
                        dx: dx as f64,
                        dy: dy as f64,
                    });
                }
                MouseEventKind::ScrollUp => {
                    self.zoom_at(0.9, Some((m.column, m.row)), area);
                    out = Some(LineChartEvent::Zoomed { factor: 0.9 });
                }
                MouseEventKind::ScrollDown => {
                    self.zoom_at(1.1, Some((m.column, m.row)), area);
                    out = Some(LineChartEvent::Zoomed { factor: 1.1 });
                }
                MouseEventKind::Moved => {
                    if let Some((x, y)) = self.term_to_data(m.column, m.row, area) {
                        out = Some(LineChartEvent::Hover { x, y });
                    }
                }
                _ => {}
            }
        } else if matches!(m.kind, MouseEventKind::Up(_)) {
            self.is_dragging = false;
        }
        Ok(out)
    }

    #[inline]
    fn term_to_data(&self, col: u16, row: u16, area: Rect) -> Option<(f64, f64)> {
        if area.width == 0 || area.height == 0 {
            return None;
        }
        if !area.contains(Position::new(col, row)) {
            return None;
        }
        let nx = (col - area.x) as f64 / area.width as f64;
        let ny = (row - area.y) as f64 / area.height as f64;
        let x = self.x_view[0] + nx * (self.x_view[1] - self.x_view[0]);
        let y = self.y_view[1] - ny * (self.y_view[1] - self.y_view[0]); // flip
        Some((x, y))
    }
}

impl ThemedWidget for LineChart {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        let color = theme.style().fg.unwrap_or(Color::Gray);

        let canvas = Canvas::default()
            .x_bounds(self.x_view)
            .y_bounds(self.y_view)
            .paint(|ctx| {
                for w in self.data.windows(2) {
                    let (x1, y1) = w[0];
                    let (x2, y2) = w[1];
                    ctx.draw(&Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        color,
                    });
                }
            });

        Widget::render(canvas, area, buf);
    }
}
