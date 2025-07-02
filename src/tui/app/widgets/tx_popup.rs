use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Duration,
};

use alloy::{
    consensus::{SignableTransaction, TxEnvelope, TxType},
    network::TxSignerSync,
    primitives::{Address, Bytes, FixedBytes},
    providers::Provider,
    rlp::{self, BytesMut, Encodable},
    rpc::types::TransactionRequest,
};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
};
use tokio::task::JoinHandle;

use crate::{
    error::FmtError,
    network::Network,
    tui::{
        app::{
            widgets::{button::Button, popup::Popup, text_scroll::TextScroll},
            SharedState,
        },
        theme::Theme,
        traits::{CustomRender, HandleResult, RectUtil},
        Event,
    },
    utils::account::AccountManager,
};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum TxStatus {
    #[default]
    NotSent,
    Signing,
    Pending(FixedBytes<32>),
    Confirmed(FixedBytes<32>),
    Failed(FixedBytes<32>),
}

#[derive(Default)]
pub struct TxPopup {
    network: Network,
    tx_req: TransactionRequest,
    text: TextScroll,
    open: bool,
    button_cursor: bool, // is cursor on the confirm button?
    tx_hash: Option<FixedBytes<32>>,
    status: TxStatus,
    send_tx_thread: Option<JoinHandle<()>>,
    watch_tx_thread: Option<JoinHandle<()>>,
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
        self.button_cursor = false;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn set_tx_req(&mut self, network: Network, tx_req: TransactionRequest) {
        self.text.text = fmt_tx_request(&network, &tx_req);
        self.network = network;
        self.tx_req = tx_req;
        self.reset();
    }

    pub fn is_not_sent(&self) -> bool {
        matches!(self.status, TxStatus::NotSent)
    }

    pub fn is_confirmed(&self) -> bool {
        matches!(self.status, TxStatus::Confirmed(_))
    }

    fn reset(&mut self) {
        self.button_cursor = false;
        self.status = TxStatus::NotSent;
        self.tx_hash = None;
        if let Some(thread) = self.send_tx_thread.take() {
            thread.abort();
        }
        if let Some(thread) = self.watch_tx_thread.take() {
            thread.abort();
        }
    }

    pub fn handle_event<F1, F2, F3, F4>(
        &mut self,
        (event, area, tr, sd, ss): (
            &crate::tui::Event,
            Rect,
            &mpsc::Sender<Event>,
            &Arc<AtomicBool>,
            &SharedState,
        ),
        mut on_tx_submit: F1,
        mut on_tx_confirm: F2,
        mut on_cancel: F3,
        mut on_esc: F4,
    ) -> crate::Result<HandleResult>
    where
        F1: FnMut(FixedBytes<32>) -> crate::Result<()>,
        F2: FnMut(FixedBytes<32>) -> crate::Result<()>,
        F3: FnMut() -> crate::Result<()>,
        F4: FnMut() -> crate::Result<()>,
    {
        let mut result = HandleResult::default();

        let r = self
            .text
            .handle_event(event, Popup::inner_area(area).block_inner().margin_down(3))?;
        result.merge(r);

        match event {
            Event::Input(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match self.status {
                        TxStatus::NotSent => match key_event.code {
                            KeyCode::Left => {
                                self.button_cursor = false;
                            }
                            KeyCode::Right => {
                                self.button_cursor = true;
                            }
                            KeyCode::Enter => {
                                if self.button_cursor {
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
            Event::TxUpdate(status) => {
                self.status = *status;

                match status {
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
            Event::TxError(_) => self.reset(),
            _ => {}
        }
        result.esc_ignores = 1;
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
            let block = Block::bordered().title("Transaction");
            let block_inner_area = block.inner(inner_area);
            block.render(inner_area, buf);

            let [text_area, button_area] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(3)])
                    .areas(block_inner_area);

            self.text.render(text_area, buf);

            let [left_area, right_area] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(button_area);

            match self.status {
                TxStatus::NotSent => {
                    Button {
                        focus: !self.button_cursor,
                        label: "Cancel",
                    }
                    .render(left_area, buf, &theme);

                    Button {
                        focus: self.button_cursor,
                        label: "Confirm",
                    }
                    .render(right_area, buf, &theme);
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

pub fn send_tx_thread(
    tx_req: &TransactionRequest,
    network: &Network,
    tr: &mpsc::Sender<Event>,
    shutdown_signal: &Arc<AtomicBool>,
    shared_state: &SharedState,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let shutdown_signal = shutdown_signal.clone();
    let sender_account = shared_state
        .current_account
        .ok_or(crate::Error::CurrentAccountNotSet)?;
    let network = network.clone();
    let tx_req = tx_req.clone();
    Ok(tokio::spawn(async move {
        let _ = match run(sender_account, network, tx_req, shutdown_signal).await {
            Ok(hash) => tr.send(Event::TxUpdate(TxStatus::Pending(hash))),
            Err(err) => tr.send(Event::TxError(err.fmt_err("TxSubmitError"))),
        };

        async fn run(
            sender_account: Address,
            network: Network,
            mut tx: TransactionRequest,
            shutdown_signal: Arc<AtomicBool>,
        ) -> crate::Result<FixedBytes<32>> {
            let provider = network.get_provider()?;

            let wallet = AccountManager::load_wallet(&sender_account)?;

            let nonce = provider.get_transaction_count(sender_account).await?;
            tx.nonce = Some(nonce);

            // Fetch chain ID
            let chain_id = provider.get_chain_id().await?;
            tx.chain_id = Some(chain_id);

            // Estimate gas fees
            let fee_estimation = provider.estimate_eip1559_fees(None).await?;
            tx.max_priority_fee_per_gas = Some(fee_estimation.max_priority_fee_per_gas);
            tx.max_fee_per_gas = Some(gm(fee_estimation.max_fee_per_gas));
            fn gm(gas_price: u128) -> u128 {
                let last_4_digits = gas_price % 10000;
                if last_4_digits != 0 {
                    gas_price - last_4_digits + 9393
                } else {
                    gas_price + 9393
                }
            }

            tx.from = Some(sender_account);
            tx.gas = Some(provider.estimate_gas(&tx).await?);
            tx.gas = tx.gas.map(|gas| gas * 110 / 100); // TODO allow to configure gas limit)

            tx.transaction_type = Some(2); // EIP-1559 transaction type

            let mut tx = tx
                .transaction_type(TxType::Eip1559.into())
                .build_typed_tx()
                .map_err(|tx| {
                    crate::Error::InternalError(format!("Tx type not specified: {tx:?}"))
                })?
                .eip1559()
                .ok_or(crate::Error::InternalErrorStr("Not 1559"))?
                .clone();

            // Sign transaction
            let signature = wallet.sign_transaction_sync(&mut tx)?;
            let tx_signed = SignableTransaction::into_signed(tx, signature);

            // Encode transaction
            let mut out = BytesMut::new();
            let tx_typed = TxEnvelope::Eip1559(tx_signed);
            tx_typed.encode(&mut out);
            let out = rlp::decode_exact::<Bytes>(out)?;

            if shutdown_signal.load(Ordering::Relaxed) {
                return Err(crate::Error::Abort("shutdown signal received"));
            }

            // Submit transaction
            let result = provider.send_raw_transaction(&out).await?;
            Ok(*result.tx_hash())
        }
    }))
}

pub fn watch_tx_thread(
    network: &Network,
    tr: &mpsc::Sender<Event>,
    shutdown_signal: &Arc<AtomicBool>,
    tx_hash: FixedBytes<32>,
) -> crate::Result<JoinHandle<()>> {
    let tr = tr.clone();
    let shutdown_signal = shutdown_signal.clone();

    let provider = network.get_provider()?;
    Ok(tokio::spawn(async move {
        loop {
            match provider.get_transaction_receipt(tx_hash).await {
                Ok(result) => {
                    if let Some(result) = result {
                        let _ = tr.send(Event::TxUpdate(if result.status() {
                            TxStatus::Confirmed(tx_hash)
                        } else {
                            TxStatus::Failed(tx_hash)
                        }));
                        break;
                    }
                }
                Err(e) => {
                    let _ = tr.send(Event::TxError(
                        crate::Error::from(e).fmt_err("TxStatusError"),
                    ));
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;

            if shutdown_signal.load(Ordering::Relaxed) {
                break;
            }
        }
    }))
}
