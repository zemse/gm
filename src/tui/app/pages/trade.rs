use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::Widget;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use tokio::task::JoinHandle;

use crate::tui::app::widgets::candle_chart::{Candle, Interval};
use crate::tui::{
    app::widgets::candle_chart::CandleChart,
    traits::{Component, HandleResult},
    Event,
};

#[derive(Default)]
pub struct TradePage {
    candle_chart: CandleChart,
    candles: Option<Vec<Candle>>,
    api_thread: Option<JoinHandle<()>>,
}

impl Component for TradePage {
    async fn exit_threads(&mut self) {
        if let Some(thread) = self.api_thread.take() {
            thread.abort();
        }
    }

    fn handle_event(
        &mut self,
        event: &crate::tui::Event,
        transmitter: &std::sync::mpsc::Sender<crate::tui::Event>,
        _shutdown_signal: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> crate::Result<crate::tui::traits::HandleResult> {
        match event {
            Event::Input(key_event) => {
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
                            let (interval, _name) = match num {
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
            Event::CandlesUpdate(candles) => {
                self.candles = Some(candles.clone());
            }
            _ => {}
        }

        if self.api_thread.is_none() {
            let tr = transmitter.clone();
            self.api_thread = Some(tokio::spawn(async move {
                // TODO query from uniswap subgraph instead
                use serde::Deserialize;
                #[allow(dead_code)]
                #[derive(Debug, Deserialize)]
                pub struct BinanceKline(
                    u64,    // open_time (ms)
                    String, // open
                    String, // high
                    String, // low
                    String, // close
                    String, // volume
                    u64,    // close_time (ms)
                    String, // quote_asset_volume
                    u64,    // number_of_trades
                    String, // taker_buy_base_volume
                    String, // taker_buy_quote_volume
                    String, // ignore
                );

                impl From<BinanceKline> for Candle {
                    fn from(kline: BinanceKline) -> Self {
                        Candle {
                            start_timestamp: kline.0 as i64,
                            open: kline.1.parse().unwrap_or(0.0),
                            high: kline.2.parse().unwrap_or(0.0),
                            low: kline.3.parse().unwrap_or(0.0),
                            close: kline.4.parse().unwrap_or(0.0),
                            end_timestamp: kline.6 as i64,
                        }
                    }
                }

                let url = "https://api.binance.com/api/v3/klines?symbol=ETHUSDT&interval=1s";
                loop {
                    match reqwest::get(url).await {
                        Ok(response) => match response.json::<Vec<BinanceKline>>().await {
                            Ok(parsed) => {
                                let candles: Vec<Candle> =
                                    parsed.into_iter().map(|kline| kline.into()).collect();
                                let _ = tr.send(Event::CandlesUpdate(candles));
                            }
                            Err(e) => eprintln!("Failed to parse response: {:?}", e),
                        },
                        Err(e) => eprintln!("HTTP request failed: {:?}", e),
                    }
                    thread::sleep(Duration::from_secs(10));
                }
            }));
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
        let mut candle_chart = CandleChart::default();
        if let Some(candles) = &self.candles {
            candle_chart.candles(candles.clone());
            candle_chart.render(area, buf);
        }

        area
    }
}
