use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::Widget;
use std::str::FromStr;
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
    candle_chart: Option<CandleChart>,
    interval: Interval,
    api_thread: Option<JoinHandle<()>>,
}

impl Component for TradePage {
    async fn exit_threads(&mut self) {
        if let Some(thread) = self.api_thread.take() {
            thread.abort();
            let _ = thread.await;
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
                if let Some(candle_chart) = &mut self.candle_chart {
                    candle_chart.handle_event(key_event)?;
                }

                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char(num)
                            if i32::from_str(&num.to_string()).is_ok()
                                && (1..=5).contains(&i32::from_str(&num.to_string()).unwrap()) =>
                        {
                            let interval = match num {
                                '1' => Interval::OneSecond,
                                '2' => Interval::FifteenMinutes,
                                '3' => Interval::OneHour,
                                '4' => Interval::OneWeek,
                                '5' => Interval::OneMonth,
                                _ => Interval::OneSecond,
                            };
                            if interval != self.interval {
                                // Do an API call and get the candles for the right interval

                                // Close the previous thread if it exists
                                if let Some(thread) = self.api_thread.take() {
                                    thread.abort();
                                }

                                // Start a new thread to fetch the candles
                                self.api_thread =
                                    Some(start_api_thread(interval, transmitter, None));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::CandlesUpdate(candles, interval) => {
                if self.candle_chart.is_none() {
                    self.candle_chart = Some(CandleChart::default())
                }

                if let Some(candle_chart) = self.candle_chart.as_mut() {
                    candle_chart.update(candles.clone(), *interval)
                }
                self.interval = *interval;
            }
            _ => {}
        }

        if self.api_thread.is_none() {
            self.api_thread = Some(start_api_thread(Interval::OneSecond, transmitter, None));
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
        if let Some(candle_chart) = &self.candle_chart {
            candle_chart.render(area, buf);
        } else {
            "Loading chart...".render(area, buf);
        }

        area
    }
}

/// Starts a thread that fetches candles from the Binance API.
/// interval - the interval for the candles.
/// transmitter - the channel to send the CandlesUpdate event.
/// query_duration - the duration for which to re-query the API.
fn start_api_thread(
    interval: Interval,
    transmitter: &std::sync::mpsc::Sender<Event>,
    query_duration: Option<Duration>,
) -> tokio::task::JoinHandle<()> {
    let tr = transmitter.clone();
    tokio::spawn(async move {
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

        let url =
            format!("https://api.binance.com/api/v3/klines?symbol=ETHUSDT&interval={interval}");
        loop {
            match reqwest::get(&url).await {
                Ok(response) => match response.json::<Vec<BinanceKline>>().await {
                    Ok(parsed) => {
                        let candles: Vec<Candle> =
                            parsed.into_iter().map(|kline| kline.into()).collect();
                        let _ = tr.send(Event::CandlesUpdate(candles, interval));
                    }
                    Err(e) => {
                        let _ = tr.send(Event::CandlesUpdateError(e));
                    }
                },
                Err(e) => {
                    let _ = tr.send(Event::CandlesUpdateError(e));
                }
            }
            tokio::time::sleep(query_duration.unwrap_or(Duration::from_secs(5))).await;
        }
    })
}
