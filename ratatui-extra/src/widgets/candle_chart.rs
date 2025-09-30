use std::fmt::Display;

use chrono::{DateTime, Local, TimeZone};
use ratatui::crossterm;
use ratatui::widgets::Widget;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    style::{Color, Style},
};

#[derive(Debug, Default)]
pub struct CandleChart {
    candles: Vec<Candle>,
    start_timestamp_g: i64,
    end_timestamp_g: i64,
    interval: Interval,
    zoom: f64,
    cursor: i64,
    y_axis_width: u16,
}
impl CandleChart {
    pub fn update(&mut self, mut new_candles: Vec<Candle>, interval: Interval) {
        let prev_end_timestamp_g = self.end_timestamp_g;

        self.start_timestamp_g = new_candles
            .iter()
            .map(|c| c.start_timestamp)
            .min()
            .unwrap_or(0);
        self.end_timestamp_g = new_candles
            .iter()
            .map(|c| c.end_timestamp)
            .max()
            .unwrap_or(0);
        new_candles.sort_by_key(|c| c.start_timestamp);
        new_candles.reverse();

        let g_max = new_candles
            .iter()
            .map(|c| c.high)
            .reduce(f64::max)
            .unwrap_or(0.0);
        let g_min = new_candles
            .iter()
            .map(|c| c.low)
            .reduce(f64::min)
            .unwrap_or(0.0);

        if interval != self.interval || self.candles.is_empty() {
            self.zoom = 1.0;
            self.cursor = self.end_timestamp_g;
        } else if self.cursor == prev_end_timestamp_g {
            self.cursor = self.end_timestamp_g;
        }

        self.candles = new_candles;
        self.interval = interval;

        self.y_axis_width =
            std::cmp::max(numeric_format(g_max).len(), numeric_format(g_min).len()) as u16 + 4;
    }

    pub fn handle_event(&mut self, key_event: &KeyEvent) {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Up => {
                    self.zoom_in();
                }
                KeyCode::Down => self.zoom_out(),
                KeyCode::Right => {
                    self.move_right();
                }
                KeyCode::Left => {
                    self.move_left();
                }
                _ => {}
            }
        }
    }

    fn zoom_in(&mut self) {
        if self.zoom + 1.0 >= 60.0 {
            self.zoom = 60.0;
        } else {
            self.zoom += 1.0;
        }
    }

    fn zoom_out(&mut self) {
        if self.zoom - 1.0 <= 1.0 {
            self.zoom = 1.0;
        } else {
            self.zoom -= 1.0;
        }
    }

    fn move_right(&mut self) {
        if self.cursor + self.interval as i64 >= self.end_timestamp_g {
            self.cursor = self.end_timestamp_g
        } else {
            self.cursor += self.interval as i64;
        }
    }

    fn move_left(&mut self) {
        // TODO widget area might be smaller than terminal size
        let chart_width = crossterm::terminal::size().unwrap_or((0, 0)).0 - self.y_axis_width;
        let start_timestamp =
            self.cursor - ((chart_width as i64 * self.interval as i64) as f64 / self.zoom) as i64;
        if start_timestamp - self.interval as i64 <= self.start_timestamp_g {
            self.cursor =
                start_timestamp + ((chart_width as i64 / self.zoom as i64) * self.interval as i64);
        } else {
            self.cursor -= self.interval as i64;
        }
    }
}

impl Widget for &CandleChart {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let y_width = self.y_axis_width;
        let chart_width = area.width - y_width - 2;
        let start_timestamp = (self.cursor / self.interval as i64
            - chart_width as i64 / self.zoom as i64)
            * self.interval as i64;
        let visible_candles_v: Vec<&Candle> = self
            .candles
            .iter()
            .filter(|c| c.end_timestamp <= self.cursor && c.start_timestamp >= start_timestamp)
            .collect::<Vec<&Candle>>();
        let visible_candles: &[&Candle] = &visible_candles_v[0..std::cmp::min(
            visible_candles_v.len(),
            chart_width as usize / self.zoom as usize,
        )];
        let y_max_all = visible_candles
            .iter()
            .map(|c| c.high)
            .reduce(f64::max)
            .unwrap_or(0.0);
        let y_min_all = visible_candles
            .iter()
            .map(|c| c.low)
            .reduce(f64::min)
            .unwrap_or(0.0);
        let y_height = area.height - 2;
        let y_scale = (y_max_all - y_min_all) / y_height as f64;
        let max_chars = std::cmp::max(
            numeric_format(y_max_all).len(),
            numeric_format(y_min_all).len(),
        );
        for i in 0..y_height {
            if i % 4 == 0 {
                let value = y_max_all - (y_scale * i as f64);
                buf.set_string(
                    area.x + chart_width,
                    area.y + i,
                    format!(" ┤ {} ", numeric_format(value)),
                    Style::default(),
                );
            } else {
                buf.set_string(
                    area.x + chart_width,
                    area.y + i,
                    format!(" │ {} ", " ".repeat(max_chars)),
                    Style::default(),
                );
            }
        }
        let mut x_axis_strings = vec![
            "─"
                .repeat(chart_width as usize)
                .chars()
                .collect::<Vec<char>>(),
            " ".repeat(chart_width as usize)
                .chars()
                .collect::<Vec<char>>(),
        ];
        let full_timestamps = (start_timestamp..=self.cursor)
            .step_by(self.interval as usize)
            .map(|t| {
                let naive = DateTime::from_timestamp_millis(t).unwrap().naive_utc();
                (t, Local.from_utc_datetime(&naive))
            })
            .collect::<Vec<(i64, DateTime<Local>)>>();
        let full_timestamps_len = full_timestamps.len();
        let mut timestamps = if full_timestamps.len() > chart_width as usize {
            full_timestamps
                .into_iter()
                .skip(full_timestamps_len - (chart_width as f64 / self.zoom) as usize)
                .collect::<Vec<(i64, DateTime<Local>)>>()
        } else {
            full_timestamps
        };

        let timestamps_len = timestamps.len();

        match timestamps_len as u64 {
            0 => {}
            1 => {
                let now = Local::now();
                let (_, last) = timestamps.last().unwrap();
                let rendered = shorted_now_string(now, *last, self.interval.render_precision());

                let written = overwrite_chars(
                    &mut x_axis_strings[1],
                    (timestamps_len - 1) as isize - (rendered.len() / 2) as isize,
                    rendered,
                    true,
                );
                if written {
                    x_axis_strings[0][timestamps_len - 1] = '┴';
                }
            }
            2.. => {
                {
                    let (_, prev) = timestamps[timestamps_len - 2];
                    let (_, now) = timestamps.last().unwrap();
                    let rendered = shorted_now_string(prev, *now, self.interval.render_precision());

                    let written = overwrite_chars(
                        &mut x_axis_strings[1],
                        chart_width as isize - (rendered.len() / 2) as isize - self.zoom as isize,
                        rendered,
                        true,
                    );
                    if written {
                        x_axis_strings[0][chart_width as usize - self.zoom as usize] = '┴';
                    }
                }

                let mut gap = 1;
                if timestamps_len > chart_width as usize {
                    gap = self.interval.render_gap() as i64;
                }
                timestamps.reverse();
                for (idx, now_t) in timestamps
                    .iter()
                    .skip(1)
                    .take((chart_width as f64 / self.zoom) as usize - 1)
                    .enumerate()
                {
                    let idx = idx + 1;
                    let (timestamp, now) = now_t;
                    let (_, prev) = if idx == 0 {
                        (0, DateTime::default())
                    } else {
                        timestamps[idx - 1]
                    };
                    if timestamp % gap != 0 {
                        continue;
                    }

                    let rendered = diff_datetime_string(prev, *now);

                    let written = overwrite_chars(
                        &mut x_axis_strings[1],
                        (chart_width as usize - ((idx + 1) * self.zoom as usize)) as isize
                            - (rendered.len() / 2) as isize
                            - 1,
                        format!(" {rendered} "),
                        false,
                    );

                    if written {
                        x_axis_strings[0]
                            [chart_width as usize - ((idx + 1) * self.zoom as usize)] = '┴';
                    }
                }
            }
        }
        buf.set_string(
            area.x + chart_width - 1,
            area.y + y_height,
            "──┘",
            Style::default(),
        );
        let x_axis_strings: Vec<String> =
            x_axis_strings.into_iter().map(String::from_iter).collect();
        for (y, string) in x_axis_strings.iter().enumerate() {
            buf.set_string(
                area.x,
                area.y + y_height + y as u16,
                string,
                Style::default(),
            );
        }
        for (i, x) in (0..std::cmp::min(
            visible_candles.len() * self.zoom as usize,
            chart_width as usize,
        ))
            .rev()
            .step_by(self.zoom as usize)
            .enumerate()
        {
            let candle = visible_candles.get(visible_candles.len() - i - 1).unwrap();
            let [y_open, y_high, y_low, y_close] = candle.calc_y(y_scale, y_min_all);
            let y_max = std::cmp::max_by(y_open, y_close, f64::total_cmp);
            let y_min = std::cmp::min_by(y_open, y_close, f64::total_cmp);
            let high_max_diff = y_high - y_max;
            let min_low_diff = y_min - y_low;
            let mut is_body = false;

            for y_chart in (0..y_height).rev() {
                let y_chart = y_chart as f64;
                let char = if y_high.ceil() >= y_chart && y_chart >= y_max.floor() {
                    if y_high - y_chart > 0.5 {
                        if high_max_diff < 0.25 {
                            is_body = true;
                            UNICODE_BODY
                        } else if high_max_diff < 0.75 {
                            if is_body {
                                is_body = true;
                                UNICODE_BODY
                            } else {
                                is_body = true;
                                UNICODE_UP
                            }
                        } else {
                            UNICODE_WICK
                        }
                    } else if y_high - y_chart >= 0. {
                        if high_max_diff < 0.25 {
                            UNICODE_HALF_BODY_BOTTOM
                        } else {
                            UNICODE_HALF_WICK_BOTTOM
                        }
                    } else {
                        UNICODE_VOID
                    }
                } else if y_max.floor() >= y_chart && y_chart >= y_min.ceil() {
                    is_body = true;
                    UNICODE_BODY
                } else if y_min.ceil() >= y_chart && y_chart >= y_low.floor() {
                    if y_low - y_chart < 0.5 {
                        if min_low_diff < 0.25 {
                            is_body = true;
                            UNICODE_BODY
                        } else if min_low_diff < 0.75 {
                            if is_body {
                                is_body = false;
                                UNICODE_DOWN
                            } else {
                                UNICODE_WICK
                            }
                        } else {
                            UNICODE_WICK
                        }
                    } else if y_low - y_chart <= 1.0 {
                        if min_low_diff < 0.25 {
                            UNICODE_HALF_BODY_TOP
                        } else {
                            UNICODE_HALF_WICK_TOP
                        }
                    } else {
                        UNICODE_VOID
                    }
                } else {
                    UNICODE_VOID
                };
                buf.set_string(
                    area.x + chart_width - x as u16 - 1,
                    area.y + y_height - y_chart as u16 - 1,
                    char,
                    candle.bear_bull(),
                );
            }
        }
    }
}
enum Precision {
    Second,
    Minute,
    Day,
}

#[repr(i64)]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Interval {
    #[default]
    OneSecond = 1000,
    FifteenMinutes = 15 * 60 * 1000,
    OneHour = 60 * 60 * 1000,
    OneWeek = 60 * 60 * 24 * 7 * 1000,
    OneMonth = 60 * 60 * 24 * 30 * 1000,
}

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let interval = match self {
            Interval::OneSecond => "1s",
            Interval::FifteenMinutes => "15m",
            Interval::OneHour => "1h",
            Interval::OneWeek => "1w",
            Interval::OneMonth => "1M",
        };
        write!(f, "{interval}")
    }
}

impl Interval {
    fn render_precision(&self) -> Precision {
        match self {
            Interval::OneSecond => Precision::Second,
            Interval::FifteenMinutes => Precision::Minute,
            Interval::OneHour => Precision::Minute,
            Interval::OneWeek => Precision::Day,
            Interval::OneMonth => Precision::Day,
        }
    }

    fn render_gap(&self) -> usize {
        match self {
            Interval::OneSecond => 30,
            Interval::FifteenMinutes => 8,
            Interval::OneHour => 12,
            Interval::OneWeek => 12,
            Interval::OneMonth => 12,
        }
    }
}

fn shorted_now_string<Tz: TimeZone>(
    prev: DateTime<Tz>,
    now: DateTime<Tz>,
    precision: Precision,
) -> String {
    let time_offset = Local;
    let prev = prev.with_timezone(&time_offset);
    let now = now.with_timezone(&time_offset);
    let prev_year = prev.format("%Y").to_string();
    let now_year = now.format("%Y").to_string();
    if prev_year != now_year {
        return match precision {
            Precision::Second => now.format("%Y/%m/%d/%m %H:%M:%S"),
            Precision::Minute => now.format("%Y/%m/%d %H:%M"),
            Precision::Day => now.format("%Y/%m/%d"),
        }
        .to_string();
    }

    let prev_date = prev.format("%m/%d").to_string();
    let now_date = now.format("%m/%d").to_string();
    if prev_date != now_date {
        return match precision {
            Precision::Second => now.format("%m/%d %H:%M:%S"),
            Precision::Minute => now.format("%m/%d %H:%M"),
            Precision::Day => now.format("%m/%d"),
        }
        .to_string();
    }

    let prev_detailed_time = prev.format("%H:%M:%S").to_string();
    let now_detailed_time = now.format("%H:%M:%S").to_string();
    if prev_detailed_time != now_detailed_time {
        return match precision {
            Precision::Second => now.format("%H:%M:%S"),
            Precision::Minute => now.format("%H:%M"),
            Precision::Day => now.format("%m/%d"),
        }
        .to_string();
    }

    String::default()
}

fn diff_datetime_string<Tz: TimeZone>(prev: DateTime<Tz>, now: DateTime<Tz>) -> String {
    let time_offset = Local;
    let prev = prev.with_timezone(&time_offset);
    let now = now.with_timezone(&time_offset);

    let prev_year = prev.format("%Y").to_string();
    let now_year = now.format("%Y").to_string();
    if prev_year != now_year {
        return now_year;
    }

    let prev_date = prev.format("%m/%d").to_string();
    let now_date = now.format("%m/%d").to_string();
    if prev_date != now_date {
        return now_date;
    }

    let prev_time = prev.format("%H:%M").to_string();
    let now_time = now.format("%H:%M").to_string();
    if prev_time != now_time {
        return now_time;
    }

    let prev_detailed_time = prev.format("%H:%M:%S").to_string();
    let now_detailed_time = now.format("%H:%M:%S").to_string();
    if prev_detailed_time != now_detailed_time {
        return now_detailed_time;
    }

    String::default()
}

fn overwrite_chars(chars: &mut Vec<char>, idx: isize, value: String, overlap: bool) -> bool {
    if chars.len() < value.len() {
        return false;
    }

    let idx = if idx < 0 {
        0
    } else if chars.len() < idx as usize + value.len() {
        chars.len() - value.len()
    } else {
        idx as usize
    };

    if !overlap {
        for &char in &chars[idx..(idx + value.len())] {
            if char != ' ' {
                // not allow overlap string value
                return false;
            }
        }
    }

    chars.splice(
        idx..(idx + value.len()),
        value.as_str().chars().collect::<Vec<char>>(),
    );

    true
}

fn numeric_format(value: f64) -> String {
    let precision = 9;
    let scale = 3;
    format!("{value:>precision$.scale$}")
}

#[derive(Clone, Debug, Default)]
pub struct Candle {
    pub start_timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub end_timestamp: i64,
}

impl Candle {
    pub fn new(
        open: f64,
        high: f64,
        close: f64,
        low: f64,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Candle {
        Candle {
            open,
            high,
            close,
            low,
            start_timestamp,
            end_timestamp,
        }
    }

    pub fn calc_y(&self, y_scale: f64, g_min: f64) -> [f64; 4] {
        [self.open, self.high, self.low, self.close].map(|v| (v - g_min) / y_scale)
    }

    pub fn bear_bull(&self) -> Color {
        if self.open <= self.close {
            Color::LightGreen
        } else {
            Color::Red
        }
    }
}

const UNICODE_VOID: &str = " ";
const UNICODE_BODY: &str = "┃";
const UNICODE_WICK: &str = "│";
const UNICODE_UP: &str = "╽";
const UNICODE_DOWN: &str = "╿";
const UNICODE_HALF_BODY_BOTTOM: &str = "╻";
const UNICODE_HALF_WICK_BOTTOM: &str = "╷";
const UNICODE_HALF_BODY_TOP: &str = "╹";
const UNICODE_HALF_WICK_TOP: &str = "╵";
