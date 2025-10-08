use std::{sync::mpsc, time::Duration};

use alloy::{
    consensus::{SignableTransaction, TxEnvelope, TxType},
    hex,
    network::TxSignerSync,
    primitives::{Address, Bytes, FixedBytes},
    providers::Provider,
    rlp::{self, BytesMut, Encodable},
    rpc::{json_rpc::ErrorPayload, types::TransactionRequest},
    transports::RpcError,
};
use gm_ratatui_extra::{
    act::Act,
    button::Button,
    extensions::{CustomRender, RectExt, ThemedWidget},
    popup::Popup,
    text_scroll::TextScroll,
    thematize::Thematize,
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
};
use serde_json::Value;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState, error::FmtError, post_handle_event::PostHandleEventActions, theme::Theme,
    AppEvent,
};
use gm_utils::{account::AccountManager, network::Network};

#[derive(Clone, Default, Debug, PartialEq)]
pub enum TxStatus {
    #[default]
    NotSent,
    Signing,
    JsonRpcError {
        message: String,
        code: i64,
        data: Option<Bytes>,
    },
    Pending(FixedBytes<32>),
    Confirmed(FixedBytes<32>),
    Failed(FixedBytes<32>),
}

#[derive(Debug)]
pub struct TxPopup {
    network: Network,
    tx_req: TransactionRequest,
    text: TextScroll,
    open: bool,

    cancel_button: Button,
    confirm_button: Button,
    is_confirm_focused: bool,

    tx_hash: Option<FixedBytes<32>>,
    status: TxStatus,
    send_tx_thread: Option<JoinHandle<()>>,
    watch_tx_thread: Option<JoinHandle<()>>,
}

impl Default for TxPopup {
    fn default() -> Self {
        Self {
            network: Network::default(),
            tx_req: TransactionRequest::default(),
            text: TextScroll::new(String::new(), true),
            open: false,
            cancel_button: Button::new("Cancel").with_success_kind(false),
            confirm_button: Button::new("Confirm").with_success_kind(true),
            is_confirm_focused: false,
            tx_hash: None,
            status: TxStatus::NotSent,
            send_tx_thread: None,
            watch_tx_thread: None,
        }
    }
}

impl TxPopup {
    pub fn new(network: Network, tx_req: TransactionRequest) -> Self {
        let mut tp = Self::default();
        tp.set_tx_req(network, tx_req);
        tp
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
        self.is_confirm_focused = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn set_tx_req(&mut self, network: Network, tx_req: TransactionRequest) {
        self.network = network;
        self.tx_req = tx_req;
        self.update_tx_req();
        self.reset();
    }

    fn update_tx_req(&mut self) {
        self.text.text = fmt_tx_request(&self.network, &self.tx_req);
    }

    pub fn is_not_sent(&self) -> bool {
        matches!(self.status, TxStatus::NotSent)
    }

    pub fn is_confirmed(&self) -> bool {
        matches!(self.status, TxStatus::Confirmed(_))
    }

    fn reset(&mut self) {
        self.is_confirm_focused = false;
        self.status = TxStatus::NotSent;
        self.tx_hash = None;
        if let Some(thread) = self.send_tx_thread.take() {
            thread.abort();
        }
        if let Some(thread) = self.watch_tx_thread.take() {
            thread.abort();
        }
    }

    pub fn handle_event<F1, F2, F3, F4, F5>(
        &mut self,
        (event, area, tr, sd, ss): (
            &AppEvent,
            Rect,
            &mpsc::Sender<AppEvent>,
            &CancellationToken,
            &SharedState,
        ),
        mut on_tx_submit: F1,
        mut on_tx_confirm: F2,
        mut on_rpc_error: F3,
        mut on_cancel: F4,
        mut on_esc: F5,
    ) -> crate::Result<PostHandleEventActions>
    where
        F1: FnMut(FixedBytes<32>) -> crate::Result<()>,
        F2: FnMut(FixedBytes<32>) -> crate::Result<()>,
        F3: FnMut(String, i64, Option<Bytes>) -> crate::Result<()>,
        F4: FnMut() -> crate::Result<()>,
        F5: FnMut() -> crate::Result<()>,
    {
        let mut result = PostHandleEventActions::default();

        self.text.handle_event(
            event.key_event(),
            Popup::inner_area(area).block_inner().margin_down(3),
        );

        match event {
            AppEvent::Input(input_event) => match input_event {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        match &self.status {
                            TxStatus::NotSent => match key_event.code {
                                KeyCode::Left => {
                                    if self.is_confirm_focused {
                                        result.ignore_left();
                                    }
                                    self.is_confirm_focused = false;
                                }
                                KeyCode::Right => {
                                    self.is_confirm_focused = true;
                                }
                                KeyCode::Enter => {
                                    if self.is_confirm_focused {
                                        self.send_tx_thread = Some(send_tx_thread(
                                            &self.tx_req,
                                            &self.network,
                                            tr,
                                            sd,
                                            ss,
                                        )?);
                                        self.status = TxStatus::Signing;
                                    } else {
                                        self.close();
                                        on_cancel()?;
                                    }
                                }
                                KeyCode::Esc => {
                                    self.close();
                                    on_esc()?;
                                }
                                _ => {}
                            },
                            TxStatus::Signing
                            | TxStatus::JsonRpcError { .. }
                            | TxStatus::Pending(_)
                            | TxStatus::Confirmed(_)
                            | TxStatus::Failed(_) =>
                            {
                                #[allow(clippy::single_match)]
                                match key_event.code {
                                    KeyCode::Esc => {
                                        self.close();
                                        on_esc()?;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Event::Mouse(_mouse_event) => {}
                _ => {}
            },
            AppEvent::TxUpdate(status) => {
                self.status = status.clone();

                match status {
                    TxStatus::JsonRpcError {
                        message,
                        code,
                        data, // TODO pass this as an error object
                    } => on_rpc_error(message.clone(), *code, data.clone())?,
                    TxStatus::Pending(tx_hash) => {
                        on_tx_submit(*tx_hash)?;

                        self.watch_tx_thread =
                            Some(watch_tx_thread(&self.network, tr, sd, *tx_hash)?);
                    }
                    TxStatus::Confirmed(tx_hash) | TxStatus::Failed(tx_hash) => {
                        on_tx_confirm(*tx_hash)?;
                    }
                    _ => {}
                }
            }
            AppEvent::TxError(_) => self.reset(),
            _ => {}
        }
        result.ignore_esc();
        Ok(result)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            Popup.render(area, buf, &theme);

            let inner_area = Popup::inner_area(area);

            let [text_area, button_area] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(inner_area);

            let text_area = text_area.block_inner();
            if theme.boxed() {
                Block::bordered()
                    .title("Transaction")
                    .style(theme.style_dim())
                    .render(inner_area, buf);
            } else {
                let block = Block::default().title("Transaction");
                block.render(inner_area, buf);
            }

            self.text.render(text_area, buf, &theme);

            let [left_area, right_area] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(button_area);

            match &self.status {
                TxStatus::NotSent => {
                    self.cancel_button.render(
                        left_area.button_center(6),
                        buf,
                        !self.is_confirm_focused,
                        &theme,
                    );

                    self.confirm_button.render(
                        right_area.button_center(7),
                        buf,
                        !self.is_confirm_focused,
                        &theme,
                    );
                }
                TxStatus::JsonRpcError {
                    message,
                    code: _,
                    data,
                } => {
                    format!("RPC Error: {} Data: {:?}", message, data)
                        .render(button_area.margin_top(1), buf);
                }
                TxStatus::Signing => {
                    "Signing and sending transaction...".render(button_area.margin_top(1), buf);
                }
                TxStatus::Pending(tx_hash) => {
                    format!("Transaction pending... Hash: {tx_hash}")
                        .render(button_area.margin_top(1), buf);
                }
                TxStatus::Confirmed(tx_hash) => {
                    [
                        format!("Transaction confirmed! Hash: {tx_hash}"),
                        "Press ESC to close".to_string(),
                    ]
                    .render(button_area.margin_top(1), buf, false);
                }
                TxStatus::Failed(tx_hash) => {
                    format!("Transaction failed! Hash: {tx_hash}")
                        .render(button_area.margin_top(1), buf);
                }
            }
        }
    }
}

fn fmt_tx_request(network: &Network, tx_req: &TransactionRequest) -> String {
    format!(
        "Network: {}\nTo: {:?}\nValue: {}\nData: {:?}\n",
        network,
        tx_req.to.unwrap_or_default(),
        tx_req.value.unwrap_or_default(),
        tx_req.input.input().unwrap_or_default()
    )
}

pub enum SendTxResult {
    Submitted(FixedBytes<32>),
    JsonRpcError(ErrorPayload),
}

pub fn send_tx_thread(
    tx_req: &TransactionRequest,
    network: &Network,
    tr: &mpsc::Sender<AppEvent>,
    shutdown_signal: &CancellationToken,
    shared_state: &SharedState,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let shutdown_signal = shutdown_signal.clone();
    let sender_account = shared_state.try_current_account()?;
    let network = network.clone();
    let tx_req = tx_req.clone();
    Ok(tokio::spawn(async move {
        let _ = match run(sender_account, network, tx_req, shutdown_signal).await {
            Ok(send_result) => tr.send(AppEvent::TxUpdate(match send_result {
                SendTxResult::Submitted(hash) => TxStatus::Pending(hash),
                SendTxResult::JsonRpcError(error_payload) => TxStatus::JsonRpcError {
                    message: error_payload.message.to_string(),
                    code: error_payload.code,
                    data: error_payload.data.and_then(|data| {
                        serde_json::from_str::<Value>(data.get())
                            .ok()
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                            .and_then(|s| hex::decode(s).ok())
                            .map(Bytes::from)
                    }),
                },
            })),
            Err(err) => tr.send(AppEvent::TxError(err.fmt_err("TxSubmitError"))),
        };

        async fn run(
            sender_account: Address,
            network: Network,
            mut tx: TransactionRequest,
            shutdown_signal: CancellationToken,
        ) -> crate::Result<SendTxResult> {
            let provider = network.get_provider()?;

            let wallet = AccountManager::load_wallet(&sender_account)?;

            let nonce = provider.get_transaction_count(sender_account).await?;
            tx.nonce = Some(nonce);

            // Fetch chain ID
            let chain_id = provider.get_chain_id().await?;
            tx.chain_id = Some(chain_id);

            tx.from = Some(sender_account);

            // Estimate gas fees
            let fee_estimation = provider.estimate_eip1559_fees().await?;
            tx.max_priority_fee_per_gas = Some(fee_estimation.max_priority_fee_per_gas);
            tx.max_fee_per_gas = Some(gm_stamp(fee_estimation.max_fee_per_gas));

            let estimate_result = provider.estimate_gas(tx.clone()).await;

            // Handle an edge case where node errors with "insufficient funds" error during revert
            let estimate_result = if estimate_result.is_err()
                && format!("{:?}", &estimate_result).contains("insufficient funds")
            {
                // re-estimate wihout gas price fields
                let mut tx_temp = tx.clone();
                tx_temp.gas_price = None;
                tx_temp.max_fee_per_gas = None;
                tx_temp.max_priority_fee_per_gas = None;

                provider.estimate_gas(tx_temp).await
            } else {
                estimate_result
            };

            // Bubble up error from estimation to client side
            let Ok(estimate) = estimate_result else {
                let err = estimate_result.err().unwrap();
                return match err {
                    RpcError::ErrorResp(payload) => Ok(SendTxResult::JsonRpcError(payload.clone())),
                    _ => Err(crate::Error::from(err)),
                };
            };

            let estimate_plus = estimate * 110 / 100; // TODO allow to configure gas limit)
            if let Some(gas) = tx.gas {
                tx.gas = Some(std::cmp::max(gas, estimate_plus));
            } else {
                tx.gas = Some(estimate_plus);
            }

            tx.transaction_type = Some(2); // EIP-1559 transaction type

            let mut tx = tx
                .transaction_type(TxType::Eip1559.into())
                .build_typed_tx()
                .map_err(|tx| crate::Error::TxTypeNotSpecified(Box::new(tx)))?
                .eip1559()
                .ok_or(crate::Error::TxTypeIsNotEip1559)?
                .clone();

            // Sign transaction
            let signature = wallet.sign_transaction_sync(&mut tx)?;
            let tx_signed = SignableTransaction::into_signed(tx, signature);

            // Encode transaction
            let mut out = BytesMut::new();
            let tx_typed = TxEnvelope::Eip1559(tx_signed);
            tx_typed.encode(&mut out);
            let out = rlp::decode_exact::<Bytes>(out)?;

            if shutdown_signal.is_cancelled() {
                return Err(crate::Error::Abort("shutdown signal received"));
            }

            // Submit transaction
            match provider.send_raw_transaction(&out).await {
                Ok(result) => Ok(SendTxResult::Submitted(*result.tx_hash())),
                Err(send_err) => match &send_err {
                    RpcError::ErrorResp(payload) => Ok(SendTxResult::JsonRpcError(payload.clone())),
                    _ => Err(crate::Error::from(send_err)),
                },
            }
        }
    }))
}

pub fn watch_tx_thread(
    network: &Network,
    tr: &mpsc::Sender<AppEvent>,
    shutdown_signal: &CancellationToken,
    tx_hash: FixedBytes<32>,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let shutdown_signal = shutdown_signal.clone();

    let provider = network.get_provider()?;
    Ok(tokio::spawn(async move {
        loop {
            tokio::select! {
                result = provider.get_transaction_receipt(tx_hash) => {
                    match result {
                        Ok(result) => {
                            if let Some(result) = result {
                                let _ = tr.send(AppEvent::TxUpdate(if result.status() {
                                    TxStatus::Confirmed(tx_hash)
                                } else {
                                    TxStatus::Failed(tx_hash)
                                }));
                                break;
                            }
                        }
                        Err(e) => {
                            let _ = tr.send(AppEvent::TxError(
                                crate::Error::from(e).fmt_err("TxStatusError"),
                            ));
                        }
                    }
                }
                _ = shutdown_signal.cancelled() => break
            };

            // TODO properly handle the wait in the first leg might need a util
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }))
}

fn gm_stamp(gas_price: u128) -> u128 {
    let last_4_digits = gas_price % 10000;
    gas_price - last_4_digits + if last_4_digits > 9393 { 19393 } else { 9393 }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gm_stamp() {
        assert_eq!(super::gm_stamp(0), 9393);
        assert_eq!(super::gm_stamp(1), 9393);
        assert_eq!(super::gm_stamp(100), 9393);
        assert_eq!(super::gm_stamp(456), 9393);

        assert_eq!(super::gm_stamp(9999), 19393);
        assert_eq!(super::gm_stamp(9998), 19393);
        assert_eq!(super::gm_stamp(9998), 19393);

        assert_eq!(super::gm_stamp(10000), 19393);
        assert_eq!(super::gm_stamp(10001), 19393);
        assert_eq!(super::gm_stamp(10002), 19393);
        assert_eq!(super::gm_stamp(10003), 19393);
        assert_eq!(super::gm_stamp(10004), 19393);

        assert_eq!(super::gm_stamp(19998), 29393);
        assert_eq!(super::gm_stamp(19999), 29393);

        assert_eq!(super::gm_stamp(1238999), 1239393);
        assert_eq!(super::gm_stamp(1239999), 1249393);
    }
}
