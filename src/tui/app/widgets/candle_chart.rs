use ratatui::widgets::Widget;

pub struct CandleChart;

impl Widget for CandleChart {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        "candle chart".render(area, buf);
    }
}
