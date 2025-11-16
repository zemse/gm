use std::sync::mpsc;

use gm_ratatui_extra::{
    act::Act,
    extensions::{MouseEventExt, RectExt},
    thematize::Thematize,
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{MouseButton, MouseEventKind},
    layout::Rect,
    text::Line,
    widgets::Widget,
};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState, post_handle_event::PostHandleEventActions, traits::Component, AppEvent,
};

pub struct Title;

struct Areas {
    gm: Rect,
    address: Rect,
    #[allow(dead_code)]
    address_shrunk: bool,
    ticker: Rect,
}

impl Title {
    fn get_data(shared_state: &SharedState) -> (String, String) {
        let account = shared_state
            .try_current_account()
            .ok()
            .map(|a| a.to_string())
            .unwrap_or("wallet".to_string());

        let display = if shared_state.online == Some(false) {
            "offline".to_string()
        } else if shared_state.config.get_testnet_mode() {
            "testnet".to_string()
        } else {
            shared_state
                .price_manager
                .get_latest_price(1)
                .as_ref()
                .map(|price| format!("ETH {:.2}", price.usd))
                .unwrap_or("loading...".to_string())
        };

        (account, display)
    }

    fn get_areas(area: Rect, shared_state: &SharedState) -> (Areas, String, String) {
        let line_area = area.margin_h(1);
        let gm_area = line_area.change_width(2);

        let line_area = line_area.margin_left(3);

        let (account, ticker) = Title::get_data(shared_state);

        let (address_area, address_shrunk) = if line_area.width
            < (account.len() + 1 + ticker.len()) as u16
        {
            let address_area = line_area.change_width(line_area.width - ticker.len() as u16 - 1);
            (address_area, true)
        } else {
            (line_area.change_width(account.len() as u16), false)
        };

        let ticker_area = Rect {
            x: line_area.x + line_area.width - ticker.len() as u16,
            y: line_area.y,
            width: ticker.len() as u16,
            height: 1,
        };

        (
            Areas {
                gm: gm_area,
                address: address_area,
                address_shrunk,
                ticker: ticker_area,
            },
            account,
            ticker,
        )
    }
}

impl Component for Title {
    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _popup_area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let mut actions = PostHandleEventActions::default();

        let (areas, account, _) = Title::get_areas(area, shared_state);

        if let Some(mouse_event) = event.mouse_event() {
            match mouse_event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    if areas.address.contains(mouse_event.position()) {
                        actions.copy_to_clipboard(account, Some(mouse_event.position()));
                    }
                }
                _ => {}
            }
        }

        Ok(actions)
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

        let (areas, account, ticker) = Title::get_areas(area, shared_state);

        Line::from("gm")
            .style(shared_state.theme.style())
            .render(areas.gm, buf);

        Line::from(account)
            .style(shared_state.theme.style())
            .render(areas.address, buf);

        Line::from(ticker)
            .style(shared_state.theme.style())
            .render(areas.ticker, buf);

        area
    }
}
