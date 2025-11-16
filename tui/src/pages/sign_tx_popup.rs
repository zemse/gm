use std::{mem, time::Duration};

use alloy::{
    consensus::{Signed, TxEip1559},
    hex::{self, ToHexExt},
    primitives::{Address, FixedBytes},
    providers::Provider,
    rpc::types::TransactionRequest,
};
use gm_common::tx_meta::TransactionMeta;
use gm_ratatui_extra::{
    act::Act,
    confirm_popup::{ConfirmPopup, ConfirmResult},
    extensions::ThemedWidget,
    popup::PopupWidget,
    text_popup::{TextPopup, TextPopupEvent},
    thematize::Thematize,
};
use ratatui::{buffer::Buffer, layout::Rect};

use crate::{post_handle_event::PostHandleEventActions, theme::Theme, AppEvent};
use gm_utils::{
    account::AccountManager,
    alloy::TxExt,
    network::Network,
    r#async::{async_once_thread, async_retry_thread, AsyncOnce},
    tx,
};

#[derive(Debug)]
pub enum SignTxEvent {
    /// When user selects cancel on the initial prompt or presses ESC
    /// during the process.
    Cancelled,

    /// User approved already and then essential fields in the tx request
    /// like gas limit, gas prices are populated.
    Built,

    /// Transaction is signed but not broadcasted yet
    Signed,

    /// Transaction is broadcasted but pending inclusion
    Broadcasted(FixedBytes<32>),

    /// Transaction is confirmed
    // TODO handle reorgs
    Confirmed(FixedBytes<32>),

    /// When transaction is confirmed with some revert or failure
    Failed(FixedBytes<32>),

    /// When transaction fails with another issue
    Error { code: i64, message: String },

    /// User presses ESC or Enter on the done popup
    Done,
}

#[derive(Debug)]
pub enum SignTxPopup {
    Closed,
    Prompt {
        confirm_popup: ConfirmPopup,
        account: Address,
        network: Network,
        tx_req: TransactionRequest,
        build_job: Option<AsyncOnce<gm_utils::Result<TxEip1559>>>,
        meta_job: Option<AsyncOnce<gm_utils::Result<TransactionMeta>>>,
    },
    PromptBuilt {
        confirm_popup: ConfirmPopup,
        confirm_text_updated: bool,
        account: Address,
        network: Network,
        tx_req: TransactionRequest,
        tx_built: TxEip1559,
        tx_meta: TransactionMeta,
    },
    Building {
        text_popup: TextPopup,
        account: Address,
        network: Network,
        tx_req: TransactionRequest,
        build_job: AsyncOnce<gm_utils::Result<TxEip1559>>,
        meta_job: AsyncOnce<gm_utils::Result<TransactionMeta>>,
    },
    Signing {
        text_popup: TextPopup,
        account: Address,
        network: Network,
        tx_req: TransactionRequest,
        tx_built: TxEip1559,
        tx_meta: TransactionMeta,
        sign_job: Option<AsyncOnce<gm_utils::Result<Signed<TxEip1559>>>>,
    },
    Sending {
        text_popup: TextPopup,
        account: Address,
        network: Network,
        tx_req: TransactionRequest,
        tx_signed: Signed<TxEip1559>,
        send_job: Option<AsyncOnce<gm_utils::Result<FixedBytes<32>>>>,
    },
    Waiting {
        text_popup: TextPopup,
        account: Address,
        network: Network,
        tx_req: TransactionRequest,
        tx_hash: FixedBytes<32>,
        wait_job: Option<AsyncOnce<gm_utils::Result<bool>>>,
    },
    Done {
        text_popup: TextPopup,
        network: Network,
        tx_hash: FixedBytes<32>,
        is_confirmed: bool,
    },
}

impl PopupWidget for SignTxPopup {
    #[track_caller]
    fn get_popup_inner(&self) -> &dyn PopupWidget {
        match self {
            SignTxPopup::Closed => unreachable!("SignTxPopup::get_popup_inner Closed"),
            SignTxPopup::Prompt { confirm_popup, .. } => confirm_popup as &dyn PopupWidget,
            SignTxPopup::PromptBuilt { confirm_popup, .. } => confirm_popup as &dyn PopupWidget,
            SignTxPopup::Building { text_popup, .. } => text_popup as &dyn PopupWidget,
            SignTxPopup::Signing { text_popup, .. } => text_popup as &dyn PopupWidget,
            SignTxPopup::Sending { text_popup, .. } => text_popup as &dyn PopupWidget,
            SignTxPopup::Waiting { text_popup, .. } => text_popup as &dyn PopupWidget,
            SignTxPopup::Done { text_popup, .. } => text_popup as &dyn PopupWidget,
        }
    }

    #[track_caller]
    fn get_popup_inner_mut(&mut self) -> &mut dyn PopupWidget {
        match self {
            SignTxPopup::Closed => unreachable!("SignTxPopup::get_popup_inner Closed"),
            SignTxPopup::Prompt { confirm_popup, .. } => confirm_popup as &mut dyn PopupWidget,
            SignTxPopup::PromptBuilt { confirm_popup, .. } => confirm_popup as &mut dyn PopupWidget,
            SignTxPopup::Building { text_popup, .. } => text_popup as &mut dyn PopupWidget,
            SignTxPopup::Signing { text_popup, .. } => text_popup as &mut dyn PopupWidget,
            SignTxPopup::Sending { text_popup, .. } => text_popup as &mut dyn PopupWidget,
            SignTxPopup::Waiting { text_popup, .. } => text_popup as &mut dyn PopupWidget,
            SignTxPopup::Done { text_popup, .. } => text_popup as &mut dyn PopupWidget,
        }
    }

    fn is_open(&self) -> bool {
        if matches!(self, SignTxPopup::Closed) {
            false
        } else {
            self.get_popup_inner().is_open()
        }
    }

    #[track_caller]
    fn open(&mut self) {
        if matches!(self, SignTxPopup::Closed) {
            unreachable!("SignTxPopup::open called when Closed, use SignTxPopup::new");
        } else {
            self.get_popup_inner_mut().open();
        }
    }
}

impl SignTxPopup {
    pub fn new(account: Address, network: Network, mut tx_req: TransactionRequest) -> Self {
        tx_req.normalize_data();

        let text = fmt_tx_request(&network, &tx_req);

        Self::Prompt {
            confirm_popup: ConfirmPopup::new("Sign", "Reject", true)
                .with_title("Transaction")
                .with_text(text)
                .with_open(true),
            account,
            network,
            tx_req,
            build_job: None,
            meta_job: None,
        }
    }

    pub fn is_not_sent(&self) -> bool {
        matches!(self, SignTxPopup::Prompt { .. })
    }

    pub fn is_confirmed(&self) -> bool {
        matches!(self, SignTxPopup::Done { .. })
    }

    fn reset(&mut self) {
        match self {
            SignTxPopup::Building { build_job, .. } => {
                build_job.thread.abort();
            }
            SignTxPopup::Signing {
                sign_job: Some(sign_job),
                ..
            } => {
                sign_job.thread.abort();
            }
            SignTxPopup::Sending {
                send_job: Some(send_job),
                ..
            } => {
                send_job.thread.abort();
            }
            SignTxPopup::Waiting {
                wait_job: Some(wait_job),
                ..
            } => {
                wait_job.thread.abort();
            }

            _ => {}
        };
    }

    pub fn handle_event(
        &mut self,
        event: &AppEvent,
        popup_area: Rect,
        actions: &mut PostHandleEventActions,
    ) -> crate::Result<Option<SignTxEvent>> {
        let mut reset = false;
        let mut result_event = None;

        if !matches!(self, SignTxPopup::Closed) {
            actions.ignore_left();
            actions.ignore_right();
        }

        match self {
            SignTxPopup::Closed => {}
            SignTxPopup::Prompt {
                confirm_popup,
                account,
                network,
                tx_req,
                build_job,
                meta_job,
            } => {
                // Start build and meta jobs in the background during the prompt
                if build_job.is_none() {
                    let account_clone = *account;
                    let network_clone = network.clone();
                    let tx_req_clone = tx_req.clone();
                    let job = async_once_thread(async move || {
                        tx::build(account_clone, network_clone, tx_req_clone).await
                    });

                    *build_job = Some(job);
                }

                if meta_job.is_none() {
                    let network_clone = network.clone();
                    let tx_req_clone = tx_req.clone();
                    let job = async_once_thread(async || {
                        tx::meta(
                            network_clone,
                            tx_req_clone,
                            TransactionMeta::default(),
                            None,
                        )
                        .await
                    });

                    *meta_job = Some(job);
                }

                if build_job.as_ref().is_some_and(|j| !j.receiver.is_empty())
                    && meta_job.as_ref().is_some_and(|j| !j.receiver.is_empty())
                {
                    let build_result = mem::take(build_job).unwrap().receiver.try_recv().unwrap();
                    let meta_result = mem::take(meta_job).unwrap().receiver.try_recv().unwrap();

                    match build_result {
                        Ok(tx_built) => {
                            // Transition to Signing state
                            *self = SignTxPopup::PromptBuilt {
                                confirm_popup: mem::take(confirm_popup),
                                confirm_text_updated: false,
                                account: *account,
                                network: mem::take(network),
                                tx_req: mem::take(tx_req),
                                tx_built,
                                tx_meta: meta_result.unwrap_or_default(),
                            };

                            return Ok(Some(SignTxEvent::Built));
                        }
                        Err(err) => {
                            let message = format!("Failed to build transaction: {}", err);

                            // Show error message in the global
                            actions.set_error(err.into());

                            // Return to prompt state
                            *self = SignTxPopup::Closed;

                            return Ok(Some(SignTxEvent::Error {
                                code: -32603, // Internal error
                                message,
                            }));
                        }
                    }
                }

                match confirm_popup.handle_event(event.input_event(), popup_area, actions)? {
                    Some(ConfirmResult::Confirmed) => {
                        // If user confirms before building, wait for build and meta jobs to be finished
                        if build_job.is_some() && meta_job.is_some() {
                            let build_job = mem::take(build_job).unwrap();
                            let meta_job = mem::take(meta_job).unwrap();

                            let confirm_popup = mem::take(confirm_popup);

                            // Transition to Building state
                            *self = SignTxPopup::Building {
                                text_popup: confirm_popup
                                    .into_text_popup()
                                    // confirm_popup is closed after selecting the confirm button
                                    .with_text("Building transaction...".to_string())
                                    .with_open(true),
                                account: *account,
                                network: mem::take(network),
                                tx_req: mem::take(tx_req),
                                build_job,
                                meta_job,
                            }
                        }
                    }
                    Some(ConfirmResult::Canceled) => {
                        confirm_popup.close();
                        result_event = Some(SignTxEvent::Cancelled);
                    }
                    None => {}
                }
            }
            Self::PromptBuilt {
                confirm_popup,
                confirm_text_updated,
                account,
                network,
                tx_req,
                tx_built,
                tx_meta,
            } => {
                if !*confirm_text_updated {
                    confirm_popup.set_text(fmt_tx_request(&network, &tx_req), true);
                    *confirm_text_updated = true;
                }

                match confirm_popup.handle_event(event.input_event(), popup_area, actions)? {
                    Some(ConfirmResult::Confirmed) => {
                        let confirm_popup = mem::take(confirm_popup);

                        // Transition to Signing state
                        *self = SignTxPopup::Signing {
                            text_popup: confirm_popup
                                .into_text_popup()
                                // confirm_popup is closed after selecting the confirm button
                                .with_open(true),
                            account: *account,
                            network: mem::take(network),
                            tx_req: mem::take(tx_req),
                            tx_built: mem::take(tx_built),
                            tx_meta: mem::take(tx_meta),
                            sign_job: None,
                        }
                    }
                    Some(ConfirmResult::Canceled) => {
                        confirm_popup.close();
                        result_event = Some(SignTxEvent::Cancelled);
                    }
                    None => {}
                }
            }
            SignTxPopup::Building {
                text_popup,
                account,
                network,
                tx_req,
                build_job,
                meta_job,
            } => {
                if let Some(TextPopupEvent::Closed) =
                    text_popup.handle_event(event.input_event(), popup_area, actions)
                {
                    // User closed the popup while building
                    result_event = Some(SignTxEvent::Cancelled);

                    reset = true;
                }

                if !build_job.receiver.is_empty() && !meta_job.receiver.is_empty() {
                    let build_result = build_job.receiver.try_recv().unwrap();
                    let meta_result = meta_job.receiver.try_recv().unwrap();

                    match build_result {
                        Ok(tx_built) => {
                            result_event = Some(SignTxEvent::Built);

                            // Transition to Signing state
                            *self = SignTxPopup::Signing {
                                text_popup: mem::take(text_popup),
                                account: *account,
                                network: mem::take(network),
                                tx_req: mem::take(tx_req),
                                tx_built,
                                tx_meta: meta_result.unwrap_or_default(),
                                sign_job: None,
                            }
                        }
                        Err(err) => {
                            result_event = Some(SignTxEvent::Error {
                                code: -32603, // Internal error
                                message: format!("Failed to build transaction: {}", err),
                            });

                            // Show error message in the global
                            actions.set_error(err.into());

                            // Return to prompt state
                            *self = SignTxPopup::Closed;
                        }
                    }
                }
            }
            SignTxPopup::Signing {
                text_popup,
                account,
                network,
                tx_req,
                tx_built,
                tx_meta,
                sign_job,
            } => {
                if let Some(TextPopupEvent::Closed) =
                    text_popup.handle_event(event.input_event(), popup_area, actions)
                {
                    // User closed the popup while signing
                    result_event = Some(SignTxEvent::Cancelled);

                    reset = true;
                }

                match sign_job {
                    None => {
                        let account_copied = *account;
                        let tx_built_clone = tx_built.clone();
                        let tx_meta = mem::take(tx_meta);

                        let job = async_once_thread(async move || {
                            // TODO will need to handle multiple wallet types in the future
                            AccountManager::sign_transaction_async(
                                account_copied,
                                tx_built_clone,
                                tx_meta,
                            )
                            .await
                        });

                        text_popup.set_text("Signing transaction...".to_string(), true);

                        *sign_job = Some(job);
                    }
                    Some(sign_job) => {
                        // Check if signing is done
                        if let Ok(result) = sign_job.receiver.try_recv() {
                            match result {
                                Ok(tx_signed) => {
                                    result_event = Some(SignTxEvent::Signed);

                                    // Transition to Sending state
                                    *self = SignTxPopup::Sending {
                                        text_popup: mem::take(text_popup),
                                        network: mem::take(network),
                                        account: *account,
                                        tx_req: mem::take(tx_req),
                                        tx_signed,
                                        send_job: None,
                                    }
                                }
                                Err(err) => {
                                    result_event = Some(SignTxEvent::Error {
                                        code: -32603, // Internal error
                                        message: format!("Failed to sign transaction: {}", err),
                                    });

                                    // Show error message in the global
                                    actions.set_error(err.into());

                                    // Return to prompt state
                                    *self = SignTxPopup::Closed
                                }
                            }
                        }
                    }
                }
            }
            SignTxPopup::Sending {
                text_popup,
                network,
                account,
                tx_req,
                tx_signed,
                send_job,
            } => {
                if let Some(TextPopupEvent::Closed) =
                    text_popup.handle_event(event.input_event(), popup_area, actions)
                {
                    // User closed the popup while sending
                    result_event = Some(SignTxEvent::Cancelled);

                    reset = true;
                }

                match send_job {
                    None => {
                        let tx_raw = tx_signed.clone().to_raw()?;
                        let provider = network.get_provider()?;
                        let job = async_once_thread(async move || {
                            provider
                                .send_raw_transaction(tx_raw.as_ref())
                                .await
                                .map(|r| *r.tx_hash())
                                .map_err(gm_utils::Error::from)
                        });

                        text_popup.set_text(
                            format!(
                                "Broadcasting transaction {tx_hash} to be confirmed...",
                                tx_hash = hex::encode_prefixed(tx_signed.hash())
                            ),
                            true,
                        );

                        *send_job = Some(job);
                    }
                    Some(send_job) => {
                        // Check if sending is done
                        if let Ok(result) = send_job.receiver.try_recv() {
                            match result {
                                Ok(tx_hash) => {
                                    *self = SignTxPopup::Waiting {
                                        text_popup: mem::take(text_popup),
                                        account: *account,
                                        network: mem::take(network),
                                        tx_req: mem::take(tx_req),
                                        tx_hash,
                                        wait_job: None,
                                    };

                                    result_event = Some(SignTxEvent::Broadcasted(tx_hash))
                                }
                                Err(err) => {
                                    result_event = Some(SignTxEvent::Error {
                                        code: -32603, // Internal error
                                        message: format!(
                                            "Failed to broadcast transaction: {}",
                                            err
                                        ),
                                    });

                                    // Show error message in the global
                                    actions.set_error(err.into());

                                    // Return to prompt state
                                    *self = SignTxPopup::Closed
                                }
                            }
                        }
                    }
                }
            }
            SignTxPopup::Waiting {
                text_popup,
                account: _,
                network,
                tx_req: _,
                tx_hash,
                wait_job,
            } => {
                if let Some(TextPopupEvent::Closed) =
                    text_popup.handle_event(event.input_event(), popup_area, actions)
                {
                    // User closed the popup while waiting
                    result_event = Some(SignTxEvent::Cancelled);

                    reset = true;
                }

                match wait_job {
                    None => {
                        let provider = network.get_provider()?;
                        let tx_hash = *tx_hash;
                        let job = async_retry_thread(
                            Duration::from_secs(2),
                            move || (provider, tx_hash),
                            |(provider, tx_hash)| {
                                // TODO this can be improved after async closures are stable
                                Box::pin(async {
                                    provider
                                        .get_transaction_receipt(*tx_hash)
                                        .await
                                        .map(|r| r.is_some())
                                        .map_err(gm_utils::Error::from)
                                })
                            },
                            |result| match result {
                                Ok(true) | Err(_) => false, // stop retrying once a receipt is found or an error occurs
                                Ok(false) => true,          // continue retrying if receipt is None
                            },
                        );

                        text_popup.set_text(
                            format!("Waiting for transaction {tx_hash} to be confirmed..."),
                            true,
                        );

                        *wait_job = Some(job);
                    }
                    Some(wait_job) => {
                        // Check if waiting is done
                        if let Ok(result) = wait_job.receiver.try_recv() {
                            match result {
                                Ok(is_confirmed) => {
                                    if is_confirmed {
                                        result_event = Some(SignTxEvent::Confirmed(*tx_hash));
                                    } else {
                                        result_event = Some(SignTxEvent::Failed(*tx_hash));
                                    }

                                    *self = SignTxPopup::Done {
                                        text_popup: mem::take(text_popup)
                                            .with_text(String::new())
                                            // Tx popup is closed with empty string
                                            .with_open(true),
                                        network: mem::take(network),
                                        tx_hash: *tx_hash,
                                        is_confirmed,
                                    };
                                }
                                Err(err) => {
                                    result_event = Some(SignTxEvent::Error {
                                        code: -32603, // Internal error
                                        message: format!("Transaction failed: {}", err),
                                    });

                                    // Show error message in the global
                                    actions.set_error(err.into());

                                    // Return to prompt state
                                    *self = SignTxPopup::Closed
                                }
                            }
                        }
                    }
                }
            }
            SignTxPopup::Done {
                text_popup,
                network,
                tx_hash,
                is_confirmed,
            } => {
                if let Some(TextPopupEvent::Closed) =
                    text_popup.handle_event(event.input_event(), popup_area, actions)
                {
                    // User closed the popup while building
                    result_event = Some(SignTxEvent::Done);

                    reset = true;
                }

                if text_popup.text().is_empty() {
                    let status_text = if *is_confirmed {
                        "Transaction is confirmed!"
                    } else {
                        "Transaction failed."
                    };

                    let url = network.get_tx_url(&tx_hash.encode_hex_with_prefix());

                    text_popup.set_text(
                        format!(
                            "{}\n\n{}",
                            status_text,
                            url.unwrap_or_else(|| format!("Tx Hash: {tx_hash}"))
                        ),
                        true,
                    );
                }
            }
        };

        if reset {
            self.reset();
        }

        Ok(result_event)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            match self {
                SignTxPopup::Closed => {}
                SignTxPopup::Prompt { confirm_popup, .. } => {
                    confirm_popup.render(area, buf, &theme);
                }
                SignTxPopup::PromptBuilt { confirm_popup, .. } => {
                    confirm_popup.render(area, buf, &theme);
                }
                SignTxPopup::Building { text_popup, .. } => {
                    text_popup.render(area, buf, &theme);
                }
                SignTxPopup::Signing { text_popup, .. } => {
                    text_popup.render(area, buf, &theme);
                }
                SignTxPopup::Sending { text_popup, .. } => {
                    text_popup.render(area, buf, &theme);
                }
                SignTxPopup::Waiting { text_popup, .. } => {
                    text_popup.render(area, buf, &theme);
                }
                SignTxPopup::Done { text_popup, .. } => {
                    text_popup.render(area, buf, &theme);
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
