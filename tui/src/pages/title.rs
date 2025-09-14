use std::sync::{atomic::AtomicBool, mpsc, Arc};

use gm_ratatui_extra::{extensions::RectExt, thematize::Thematize};
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::Line, widgets::Widget};

use crate::{
    app::SharedState,
    traits::{Actions, Component},
    Event,
};

pub struct Title;

impl Component for Title {
    fn handle_event(
        &mut self,
        _event: &crate::Event,
        _area: Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let result = Actions::default();
        Ok(result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        buf.set_style(area, shared_state.theme.block());
        let area = area.margin_h(1);

        let welcome_string = format!(
            "gm {account}",
            account = shared_state
                .current_account
                .map(|a| a.to_string())
                .unwrap_or("wallet".to_string())
        );

        Line::from(welcome_string)
            // TODO change the name of .block
            .style(shared_state.theme.block())
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
            .style(shared_state.theme.block())
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
