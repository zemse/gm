use std::{
    fmt::Debug,
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Duration,
};

use alloy::{
    consensus::{SignableTransaction, TxEip1559, TxEnvelope},
    network::TxSignerSync,
    primitives::{Address, Bytes, FixedBytes, TxKind, U256},
    providers::Provider,
    rlp::{BytesMut, Encodable},
};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::Line, widgets::Widget};
use tokio::task::JoinHandle;

use crate::{
    actions::account::load_wallet,
    disk::DiskInterface,
    error::FmtError,
    network::{Network, NetworkStore},
    tui::{
        app::{widgets::button::Button, Focus, SharedState},
        events::Event,
        traits::{Component, CustomRender, HandleResult, RectUtil},
    },
};

#[derive(Clone, Default, Debug)]
pub enum TxStatus {
    #[default]
    NotSent,
    Signing,
    Pending(FixedBytes<32>),
    Confirmed(FixedBytes<32>),
    Failed(FixedBytes<32>),
}

#[derive(Default)]
pub struct TransactionPage {
    pub network: Network,
    pub to: TxKind,
    pub calldata: Bytes,
    pub value: U256,
    pub tx_hash: Option<FixedBytes<32>>,
    pub status: TxStatus,
    pub send_tx_thread: Option<JoinHandle<()>>,
    pub watch_tx_thread: Option<JoinHandle<()>>,
}

impl TransactionPage {
    pub fn new(
        network_name: &str,
        to: TxKind,
        calldata: Bytes,
        value: U256,
    ) -> crate::Result<Self> {
        let network_store = NetworkStore::load();
        let network = network_store
            .get_by_name(network_name)
            .ok_or(crate::Error::NetworkNotFound(network_name.to_string()))?;

        Ok(Self {
            network,
            to,
            calldata,
            value,
            tx_hash: None,
            status: TxStatus::NotSent,
            send_tx_thread: None,
            watch_tx_thread: None,
        })
    }

    fn send_tx_thread(
        &self,
        tr: &mpsc::Sender<Event>,
        shutdown_signal: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> crate::Result<JoinHandle<()>> {
        let tr = tr.clone();
        let shutdown_signal = shutdown_signal.clone();
        let sender_account = shared_state
            .current_account
            .ok_or(crate::Error::InternalErrorStr("No current account"))?;
        let network = self.network.clone();
        let to = self.to;
        let calldata = self.calldata.clone();
        let value = self.value;

        Ok(tokio::spawn(async move {
            let _ = match run(
                sender_account,
                network,
                to,
                calldata,
                value,
                shutdown_signal,
            )
            .await
            {
                Ok(hash) => tr.send(Event::TxSubmitResult(hash)),
                Err(err) => tr.send(Event::TxSubmitError(err.fmt_err("TxSubmitError"))),
            };

            async fn run(
                sender_account: Address,
                network: Network,
                to: TxKind,
                calldata: Bytes,
                value: U256,
                shutdown_signal: Arc<AtomicBool>,
            ) -> crate::Result<FixedBytes<32>> {
                let provider = network.get_provider()?;

                let wallet = load_wallet(sender_account)?;

                let mut tx = TxEip1559 {
                    to,
                    input: calldata,
                    value,
                    ..Default::default()
                };

                let nonce = provider.get_transaction_count(sender_account).await?;
                tx.nonce = nonce;

                // Fetch chain ID
                let chain_id = provider.get_chain_id().await?;
                tx.chain_id = chain_id;

                tx.gas_limit = 51_000;

                // Estimate gas fees
                let fee_estimation = provider.estimate_eip1559_fees(None).await?;
                tx.max_priority_fee_per_gas = fee_estimation.max_priority_fee_per_gas;
                tx.max_fee_per_gas = gm(fee_estimation.max_fee_per_gas);
                fn gm(gas_price: u128) -> u128 {
                    let last_4_digits = gas_price % 10000;
                    if last_4_digits != 0 {
                        gas_price - last_4_digits + 9393
                    } else {
                        gas_price + 9393
                    }
                }

                // Sign transaction
                let signature = wallet.sign_transaction_sync(&mut tx)?;
                let tx_signed = SignableTransaction::into_signed(tx, signature);

                // Encode transaction
                let mut out = BytesMut::new();
                let tx_typed = TxEnvelope::Eip1559(tx_signed);
                tx_typed.encode(&mut out);
                let out = &out[2..];

                if shutdown_signal.load(Ordering::Relaxed) {
                    return Err(crate::Error::Abort("shutdown signal received"));
                }

                // Submit transaction
                let result = provider.send_raw_transaction(out).await?;
                Ok(*result.tx_hash())
            }
        }))
    }

    fn watch_tx_thread(
        &mut self,
        tr: &mpsc::Sender<Event>,
        shutdown_signal: &Arc<AtomicBool>,
        tx_hash: FixedBytes<32>,
    ) -> crate::Result<JoinHandle<()>> {
        let tr = tr.clone();
        let shutdown_signal = shutdown_signal.clone();

        let provider = self.network.get_provider()?;
        Ok(tokio::spawn(async move {
            loop {
                match provider.get_transaction_receipt(tx_hash).await {
                    Ok(result) => {
                        if result.is_some() {
                            let _ = tr.send(Event::TxStatus(TxStatus::Confirmed(tx_hash)));
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tr.send(Event::TxStatusError(
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
}

impl Component for TransactionPage {
    fn exit_threads(&mut self) -> impl Future<Output = ()> {
        let send_tx_thread = self.send_tx_thread.take();
        let watch_tx_thread = self.watch_tx_thread.take();

        async move {
            if let Some(thread) = send_tx_thread {
                thread.abort();
                thread.await.unwrap();
            }
            if let Some(thread) = watch_tx_thread {
                thread.await.unwrap();
            }
        }
    }

    fn handle_event(
        &mut self,
        event: &Event,
        tr: &mpsc::Sender<Event>,
        sd: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        match event {
            Event::Input(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    #[allow(clippy::single_match)]
                    match key_event.code {
                        KeyCode::Enter => {
                            if self.send_tx_thread.is_none() {
                                // Handle sending transaction
                                self.status = TxStatus::Signing;
                                self.send_tx_thread =
                                    Some(self.send_tx_thread(tr, sd, shared_state).inspect_err(
                                        |_| {
                                            self.status = TxStatus::NotSent;
                                        },
                                    )?);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::TxSubmitError(_) => {
                self.status = TxStatus::NotSent;
                if let Some(thread) = self.send_tx_thread.take() {
                    thread.abort();
                }
            }
            Event::TxSubmitResult(hash) => {
                self.send_tx_thread = None;
                self.tx_hash = Some(*hash);

                self.status = TxStatus::Pending(*hash);
                if self.watch_tx_thread.is_none() {
                    self.watch_tx_thread = Some(self.watch_tx_thread(tr, sd, *hash)?);
                }
            }

            Event::TxStatus(result) => {
                self.watch_tx_thread = None;

                self.status = result.clone();
            }
            _ => {}
        }

        Ok(HandleResult::default())
    }

    fn render_component(&self, mut area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        Line::from("Transaction Review").bold().render(area, buf);
        area = area.consume_height(2);

        let area_1 = [
            format!("Network: {}", self.network),
            format!("To: {:?}", self.to),
            format!("Calldata: {:?}", self.calldata),
            format!("Value: {:?}", self.value),
        ]
        .render(area, buf, false);
        area = area.consume_height(area_1.height);

        match self.status {
            TxStatus::NotSent => Button {
                label: "Send Transaction",
                focus: shared_state.focus == Focus::Main,
            }
            .render(area, buf),

            TxStatus::Signing => {
                "Signing and sending transaction...".render(area, buf);
            }
            TxStatus::Pending(tx_hash) => {
                format!("Transaction pending... Hash: {}", tx_hash).render(area, buf);
            }
            TxStatus::Confirmed(tx_hash) => {
                format!("Transaction confirmed! Hash: {}", tx_hash).render(area, buf);
            }
            TxStatus::Failed(tx_hash) => {
                format!("Transaction failed! Hash: {}", tx_hash).render(area, buf);
            }
        }

        area
    }
}
