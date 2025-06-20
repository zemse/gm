use std::sync::{atomic::AtomicBool, mpsc, Arc};

use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::Line, widgets::Widget};

use crate::tui::{
    app::SharedState,
    traits::{Component, HandleResult, RectUtil},
    Event,
};

pub struct Title;

impl Component for Title {
    fn handle_event(
        &mut self,
        _event: &crate::tui::Event,
        _area: Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        let result = HandleResult::default();
        Ok(result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        let area = area.margin_h(1);

        let welcome_string = format!(
            "gm {account}",
            account = shared_state
                .current_account
                .map(|a| a.to_string())
                .unwrap_or("wallet".to_string())
        );

        Line::from(welcome_string)
            .style(&shared_state.theme)
            .bold()
            .render(area, buf);

        let display = if shared_state.online == Some(false) {
            "offline".to_string()
        } else if shared_state.testnet_mode {
            "testnet".to_string()
        } else {
            shared_state
                .eth_price
                .as_ref()
                .map(|price| format!("ETH {price}"))
                .unwrap_or("loading...".to_string())
        };

        Line::from(display)
            .style(&shared_state.theme)
            .bold()
            .right_aligned()
            .render(area, buf);

        // let pkg_version = env!("CARGO_PKG_VERSION");
        // Line::from(
        //     // format!(
        //     // "version {pkg_version}{}",
        //     match self.online {
        //         Some(true) => format!(),
        //         Some(false) => "offline",
        //         None => "",
        //     }, // )
        // )
        // .bold()
        // .right_aligned()
        // .render(area, buf);

        area
    }
}
