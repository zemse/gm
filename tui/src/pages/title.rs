use std::sync::mpsc;

use gm_ratatui_extra::{extensions::RectExt, thematize::Thematize};
use ratatui::{buffer::Buffer, layout::Rect, text::Line, widgets::Widget};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState, post_handle_event::PostHandleEventActions, traits::Component, AppEvent,
};

pub struct Title;

impl Component for Title {
    fn handle_event(
        &mut self,
        _event: &AppEvent,
        _area: Rect,
        _popup_area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let result = PostHandleEventActions::default();
        Ok(result)
    }

    fn render_component(
        &self,
        area: Rect,
        _popup_area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        buf.set_style(area, shared_state.theme.style_dim());
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
            .style(shared_state.theme.style())
            .render(area, buf);

        let display = if shared_state.online == Some(false) {
            "offline".to_string()
        } else if shared_state.testnet_mode {
            "testnet".to_string()
        } else {
            shared_state
                .price_manager
                .get_latest_price(1)
                .as_ref()
                .map(|price| format!("ETH {:.2}", price.usd))
                .unwrap_or("loading...".to_string())
        };

        Line::from(display)
            .style(shared_state.theme.style())
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
