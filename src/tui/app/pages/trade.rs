use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::Widget;
use std::str::FromStr;

use crate::tui::app::widgets::candle_chart::{Candle, Interval};
use crate::tui::{
    app::widgets::candle_chart::CandleChart,
    traits::{Component, HandleResult},
    Event,
};

#[derive(Default)]
pub struct TradePage {
    candle_chart: CandleChart,
}

impl Component for TradePage {
    fn handle_event(
        &mut self,
        event: &crate::tui::Event,
        _transmitter: &std::sync::mpsc::Sender<crate::tui::Event>,
        _shutdown_signal: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> crate::Result<crate::tui::traits::HandleResult> {
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Up => {
                        self.candle_chart.zoom_in();
                    }
                    KeyCode::Down => self.candle_chart.zoom_out(),
                    KeyCode::Right => {
                        self.candle_chart.move_right();
                    }
                    KeyCode::Left => {
                        self.candle_chart.move_left();
                    }
                    KeyCode::Char(num)
                        if i32::from_str(&num.to_string()).is_ok()
                            && (1..=5).contains(&i32::from_str(&num.to_string()).unwrap()) =>
                    {
                        let (interval, name) = match num {
                            '1' => (Interval::OneSecond, "1s"),
                            '2' => (Interval::FifteenMinutes, "15m"),
                            '3' => (Interval::OneHour, "1h"),
                            '4' => (Interval::OneWeek, "1w"),
                            '5' => (Interval::OneMonth, "1M"),
                            _ => (Interval::OneSecond, "1s"),
                        };
                        if interval != self.candle_chart.interval() {
                            //Do an API call and get the candles for the right interval
                            let candles: Vec<Candle> = vec![];
                            self.candle_chart.candles(candles);
                            self.candle_chart.set_interval(interval);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(HandleResult::default())
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        _shared_state: &crate::tui::app::SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        let candles: Vec<Candle> = vec![];
        let mut candle_chart = CandleChart::default();
        candle_chart.candles(candles);
        candle_chart.render(area, buf);

        area
    }
}
