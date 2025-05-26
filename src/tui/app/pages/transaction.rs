use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

use alloy::{
    consensus::{SignableTransaction, TxEip1559, TxEnvelope},
    network::TxSignerSync,
    primitives::{Address, Bytes, FixedBytes, TxKind, U256},
    providers::Provider,
    rlp::{BytesMut, Encodable},
};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tokio::task::JoinHandle;

use crate::{
    actions::account::load_wallet,
    disk::DiskInterface,
    network::{Network, NetworkStore},
    tui::{
        app::{widgets::button::Button, Focus, SharedState},
        events::Event,
        traits::{Component, CustomRender, HandleResult, RectUtil},
    },
};

#[derive(Default)]
pub struct TransactionPage {
    pub network: Network,
    pub to: TxKind,
    pub calldata: Bytes,
    pub value: U256,
    pub send_tx_thread: Option<JoinHandle<()>>,
    pub tx_hash: Option<FixedBytes<32>>,
}

impl TransactionPage {
    pub fn new(network: &str, to: TxKind, calldata: Bytes, value: U256) -> crate::Result<Self> {
        let network_store = NetworkStore::load();
        let network = network_store
            .get_by_name(network)
            .ok_or(crate::Error::InternalErrorStr("Network not found"))?;

        Ok(Self {
            network,
            to,
            calldata,
            value,
            send_tx_thread: None,
            tx_hash: None,
        })
    }

    fn start_send_tx_thread(
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
                Ok(hash) => tr.send(Event::TxResult(hash)),
                Err(err) => tr.send(Event::TxError(err.to_string())),
            };

            async fn run(
                sender_account: Address,
                network: Network,
                to: TxKind,
                calldata: Bytes,
                value: U256,
                shutdown_signal: Arc<AtomicBool>,
            ) -> crate::Result<FixedBytes<32>> {
                let provider = network.get_provider();

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
                tx.max_fee_per_gas = fee_estimation.max_fee_per_gas;

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
}

impl Component for TransactionPage {
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
                            // Handle sending transaction
                            self.send_tx_thread =
                                Some(self.start_send_tx_thread(tr, sd, shared_state)?);
                        }
                        _ => {}
                    }
                }
            }
            Event::TxResult(hash) => {
                self.send_tx_thread = None;
                self.tx_hash = Some(*hash);
            }
            _ => {}
        }

        Ok(HandleResult::default())
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, shared_state: &SharedState) -> Rect
    where
        Self: Sized,
    {
        let area_1 = [
            format!("Network: {}", self.network),
            format!("To: {:?}", self.to),
            format!("Calldata: {:?}", self.calldata),
            format!("Value: {:?}", self.value),
        ]
        .render(area, buf, true);

        let [_, next_area] = area.split_vertical(area_1.height);

        if let Some(tx_hash) = &self.tx_hash {
            format!("Transaction sent! Hash: {}", tx_hash).render(next_area, buf);
        } else {
            Button {
                label: "Send Transaction",
                focus: shared_state.focus == Focus::Main,
            }
            .render(next_area, buf);
        }

        area
    }
}
