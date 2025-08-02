use std::sync::{atomic::AtomicBool, mpsc::Sender, Arc};

use alloy::primitives::U256;
use data3::oneinch;
use fusion_plus_sdk::{
    api::Api as FusionPlusApi, chain_id::ChainId, multichain_address::MultichainAddress,
    quote::QuoteRequest,
};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    disk::Config,
    tui::{
        app::SharedState,
        traits::{Component, HandleResult, RectUtil},
        Event,
    },
};

pub struct FusionPlusPage {
    pub src_chain_id: ChainId,
    pub src_token: MultichainAddress,
    pub dst_chain_id: ChainId,
    pub dst_token: MultichainAddress,
    pub dst_address: MultichainAddress,
    pub src_amount: U256,

    pub est_amounts_thread: Option<tokio::task::JoinHandle<()>>,
}

impl Component for FusionPlusPage {
    fn handle_event(
        &mut self,
        event: &Event,
        area: Rect,
        transmitter: &Sender<Event>,
        shutdown_signal: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        if self.est_amounts_thread.is_none() {
            let tr = transmitter.clone();
            let src_amount = self.src_amount.clone();
            self.est_amounts_thread = Some(tokio::spawn(async move {
                let Ok(oneinch_api_key) = Config::oneinch_api_key() else {
                    let _ = tr.send(Event::FusionPlusError(
                        "OneInch API key not found in config".to_string(),
                    ));
                    return;
                };
                let api = FusionPlusApi::new("https://api.1inch.dev/fusion-plus", oneinch_api_key);

                // api.get_quote(&QuoteRequest::new(
                //     ChainId::Arbitrum,
                //     ChainId::Optimism,
                //     usdc(ChainId::Arbitrum),
                //     usdc(ChainId::Optimism),
                //     U256::from(1e6),
                //     true,
                //     wallet.address(),
                // ));
            }));
        }

        Ok(HandleResult::default())
    }

    fn render_component(
        &self,
        area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        if let Ok(remaining_area) = area.consume_height(1) {
            "temp".render(area, buf);
            remaining_area
        } else {
            area
        }
    }
}
