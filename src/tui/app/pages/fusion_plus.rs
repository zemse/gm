use std::{
    fmt::Display,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc},
    time::Duration,
};

use alloy::primitives::{keccak256, utils::format_units, B256, U256, U64};

use fusion_plus_sdk::{
    addresses::get_limit_order_contract_address,
    api::{
        types::{OrderStatus, OrderStatusResponse},
        Api as FusionPlusApi,
    },
    chain_id::ChainId,
    constants::UINT_256_MAX,
    cross_chain_order::{CrossChainOrderParams, Fee, PreparedOrder},
    hash_lock::HashLock,
    limit::eip712::{get_limit_order_v4_domain, LimitOrderV4},
    multichain_address::MultichainAddress,
    quote::{QuoteRequest, QuoteResult},
    relayer_request::RelayerRequest,
    utils::{alloy::ERC20, random::get_random_bytes32},
};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    disk::Config,
    network::{Network, Token},
    tui::{
        app::{
            widgets::{
                confirm_popup::ConfirmPopup, sign_712_popup::Sign712Popup, text_scroll::TextScroll,
                tx_popup::TxPopup,
            },
            SharedState,
        },
        traits::{Component, CustomRender, HandleResult, RectUtil},
        Event,
    },
    utils::Provider,
};

#[allow(dead_code)]
enum State {
    Idle,
    Quoting,
    CheckingAllowance,
    ApprovingTokens,
    PreparingOrder,
    SigningOrder,
    SubmittingOrder,
    WaitingForFinality,
    PublishingSecret,
    WaitingForUnlock,
    Done,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            State::Idle => "Starting",
            State::Quoting => "Getting quote from Relayer",
            State::CheckingAllowance => "Checking Allowance",
            State::ApprovingTokens => "Approving Tokens",
            State::PreparingOrder => "Preparing Order",
            State::SigningOrder => "Signing Order",
            State::SubmittingOrder => "Submitting Order",
            State::WaitingForFinality => "Waiting for Finality",
            State::PublishingSecret => "Publishing Secret",
            State::WaitingForUnlock => "Waiting for Unlock",
            State::Done => "Completed!",
        };
        write!(f, "{str}")
    }
}

fn spawn_quoter_thread(
    tr: Sender<Event>,
    quote_request: QuoteRequest,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let Ok(oneinch_api_key) = Config::oneinch_api_key() else {
            let _ = tr.send(Event::FusionPlusError(
                "OneInch API key not found in config".to_string(),
            ));
            return;
        };
        let api = FusionPlusApi::new("https://api.1inch.dev/fusion-plus", oneinch_api_key);

        let result = api.get_quote(&quote_request).await;

        match result {
            Ok(quote_result) => {
                let _ = tr.send(Event::FusionPlusQuoteResult(Box::new(quote_result)));
            }
            Err(err) => {
                let _ = tr.send(Event::FusionPlusError(err.to_string()));
            }
        };
    })
}

fn spawn_allowance_check_thread(
    tr: Sender<Event>,
    src_token: MultichainAddress,
    src_amount: U256,
    src_chain_id: ChainId,
    src_provider: Provider,
    maker_address: MultichainAddress,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let src_erc20 = ERC20::new(src_token.as_raw(), src_provider);
        let spender = get_limit_order_contract_address(src_chain_id);

        loop {
            let allowance = src_erc20
                .allowance(maker_address.as_raw(), spender.as_raw())
                .call()
                .await;

            match allowance {
                Ok(allowance) => {
                    let _ = tr.send(Event::FusionPlusAllowanceResult {
                        src_chain_id,
                        src_token,
                        allowance,
                    });

                    if allowance >= src_amount {
                        break;
                    }
                }
                Err(err) => {
                    let _ = tr.send(Event::FusionPlusError(err.to_string()));
                }
            };

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    })
}

fn spawn_submit_order_thread(
    tr: Sender<Event>,
    rr: RelayerRequest,
    secrets: Vec<B256>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let Ok(oneinch_api_key) = Config::oneinch_api_key() else {
            let _ = tr.send(Event::FusionPlusError(
                "OneInch API key not found in config".to_string(),
            ));
            return;
        };
        let api = FusionPlusApi::new("https://api.1inch.dev/fusion-plus", oneinch_api_key);

        let order_hash = rr.order_hash();
        match api.submit_order(rr).await {
            Ok(_) => {
                let _ = tr.send(Event::FusionPlusOrderSubmitted);
            }
            Err(err) => {
                let _ = tr.send(Event::FusionPlusError(err.to_string()));
            }
        }

        loop {
            // let mut done = false;
            match api.get_ready_to_accept_secret_fills(&order_hash).await {
                Ok(read) => {
                    for fill in read.fills {
                        println!("Fill {fill:#?}");
                        api.submit_secret(&order_hash, &secrets[fill.idx as usize])
                            .await
                            .unwrap();
                        // done = true;
                    }
                }
                Err(err) => {
                    let _ = tr.send(Event::FusionPlusError(err.to_string()));
                }
            }

            // if done {
            //     break;
            // }

            match api.get_order_status(order_hash).await {
                Ok(order_status) => {
                    let status = order_status.status;
                    let _ = tr.send(Event::FusionPlusOrderStatus(Box::new(order_status)));
                    if matches!(
                        status,
                        OrderStatus::Executed
                            | OrderStatus::Expired
                            | OrderStatus::Cancelled
                            | OrderStatus::Refunded
                    ) {
                        let _ = tr.send(Event::FusionPlusOrderDone);
                        break;
                    }
                }
                Err(err) => {
                    let _ = tr.send(Event::FusionPlusError(err.to_string()));
                }
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    })
}

fn prepare_secrets(quote_result: &QuoteResult) -> crate::Result<(Vec<B256>, Vec<B256>, HashLock)> {
    let secrets_count = quote_result.recommended_preset().secrets_count;
    let secrets: Vec<B256> = (0..secrets_count).map(|_| get_random_bytes32()).collect();
    let secret_hashes: Vec<B256> = secrets.iter().map(HashLock::hash_secret).collect();

    let hash_lock = if secrets_count == 1 {
        HashLock::for_single_fill(&secrets[0])
    } else {
        HashLock::for_multiple_fills(
            secrets
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let mut encoded = [0u8; 36];
                    encoded[0..4].copy_from_slice(U64::from(i).to_be_bytes::<4>().as_ref());
                    encoded[4..36].copy_from_slice(s.as_ref());

                    keccak256(encoded)
                })
                .collect(),
        )?
    };
    Ok((secret_hashes, secrets, hash_lock))
}

pub struct FusionPlusPage {
    src_chain: Network,
    src_token: Token,
    src_amount: U256,
    dst_chain: Network,
    dst_token: Token,
    dst_address: MultichainAddress,

    quote_request: QuoteRequest,
    quote_thread: Option<tokio::task::JoinHandle<()>>,
    quote_result: Option<QuoteResult>,
    quote_confirm_popup: ConfirmPopup,

    allowance_thread: Option<tokio::task::JoinHandle<()>>,
    allowance_result: Option<U256>,
    approve_tx_popup: TxPopup,

    secrets: Option<Vec<B256>>,
    secret_hashes: Option<Vec<B256>>,
    prepared_order: Option<PreparedOrder>,
    sign_popup: Sign712Popup<LimitOrderV4>,

    submit_order_thread: Option<tokio::task::JoinHandle<()>>,
    order_status: Option<OrderStatusResponse>,
    display: TextScroll,

    state: State,
}

impl FusionPlusPage {
    pub fn new(
        src_chain: Network,
        src_token: Token,
        src_amount: U256,
        dst_chain: Network,
        dst_token: Token,
        dst_address: MultichainAddress,
    ) -> Self {
        let src_chain_id = ChainId::from_u32(src_chain.chain_id);
        let dst_chain_id = ChainId::from_u32(dst_chain.chain_id);

        let quote_request = QuoteRequest::new(
            src_chain_id,
            dst_chain_id,
            src_token.contract_address,
            dst_token.contract_address,
            src_amount,
            true,
            MultichainAddress::ZERO,
        );

        let eip712_domain = get_limit_order_v4_domain(src_chain_id);

        FusionPlusPage {
            src_chain,
            src_token,
            src_amount,
            dst_chain,
            dst_token,
            dst_address,

            quote_request,
            quote_thread: None,
            quote_result: None,
            quote_confirm_popup: ConfirmPopup::new(
                "Quote for Fusion Plus Transfer",
                String::new(),
                "Accept",
                "Cancel",
            ),

            allowance_thread: None,
            allowance_result: None,
            approve_tx_popup: TxPopup::default(),

            secrets: None,
            secret_hashes: None,
            prepared_order: None,
            sign_popup: Sign712Popup::new(eip712_domain),

            submit_order_thread: None,
            order_status: None,
            display: TextScroll::default(),

            state: State::Idle,
        }
    }
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
        let mut result = HandleResult::default();

        if self.quote_thread.is_none() {
            if let Some(current_account) = &shared_state.current_account {
                self.state = State::Quoting;
                self.quote_request.maker_address = *current_account;
                self.quote_thread = Some(spawn_quoter_thread(
                    transmitter.clone(),
                    self.quote_request.clone(),
                ));
            }
        }

        match event {
            Event::FusionPlusQuoteResult(quote_result) => {
                self.quote_result = Some(*quote_result.clone());
                *self.quote_confirm_popup.text_mut() = format!(
                    "Input tokens {src_parsed} {src_symbol}:\nEstimated output tokens {dst_parsed} {dst_symbol}.",
                    src_parsed = format_units(self.src_amount, self.src_token.decimals)?,
                    src_symbol = self.src_token.symbol,
                    dst_parsed = format_units(
                        quote_result.recommended_preset().auction_end_amount,
                        self.dst_token.decimals
                    )?,
                    dst_symbol = self.dst_token.symbol,
                );
                self.quote_confirm_popup.open();
            }
            Event::FusionPlusAllowanceResult {
                src_chain_id,
                src_token,
                allowance,
            } => {
                if *src_chain_id as u32 == self.src_chain.chain_id
                    && src_token.as_raw() == self.src_token.contract_address.as_raw()
                {
                    self.allowance_result = Some(*allowance);

                    if *allowance < self.src_amount {
                        let src_erc20 =
                            ERC20::new(src_token.as_raw(), self.src_chain.get_provider()?);
                        let spender = get_limit_order_contract_address(*src_chain_id);
                        let call = src_erc20
                            .approve(spender.as_raw(), UINT_256_MAX)
                            .into_transaction_request();

                        if !self.approve_tx_popup.is_open() {
                            self.approve_tx_popup
                                .set_tx_req(self.src_chain.clone(), call);
                            self.approve_tx_popup.open();
                        }
                    } else if let Some(quote_result) = &self.quote_result {
                        self.state = State::PreparingOrder;
                        let (secret_hashes, secrets, hash_lock) = prepare_secrets(quote_result)?;
                        self.secrets = Some(secrets);
                        self.secret_hashes = Some(secret_hashes.clone());

                        let order = PreparedOrder::from_quote(
                            &self.quote_request,
                            quote_result,
                            CrossChainOrderParams {
                                dst_address: self.dst_address.without_chain_id(),
                                hash_lock,
                                secret_hashes,
                                fee: Some(Fee {
                                    taking_fee_bps: 100,
                                    taking_fee_receiver: MultichainAddress::ZERO,
                                }),
                                preset: None,
                            },
                        )?;
                        self.prepared_order = Some(order.clone());
                        self.sign_popup.set_data_struct(order.to_v4());
                        self.state = State::SigningOrder;
                        self.sign_popup.open();
                    }
                }
            }
            Event::FusionPlusOrderSubmitted => {
                self.state = State::WaitingForFinality;
            }
            Event::FusionPlusOrderStatus(order_status) => {
                self.order_status = Some(*order_status.clone());
                self.display.text = format!("{order_status:#?}");
            }
            Event::FusionPlusOrderDone => self.state = State::Done,
            _ => {}
        }

        self.display.handle_event(event, area.margin_top(2))?;

        let confirm_popup_result = self.quote_confirm_popup.handle_event(
            event,
            area,
            || {
                self.state = State::CheckingAllowance;
                self.allowance_thread = Some(spawn_allowance_check_thread(
                    transmitter.clone(),
                    self.src_token.contract_address,
                    self.src_amount,
                    ChainId::from_u32(self.src_chain.chain_id),
                    self.src_chain.get_provider()?,
                    shared_state.try_current_account()?,
                ));

                Ok(())
            },
            || Ok(()),
        )?;
        result.merge(confirm_popup_result);

        let approve_tx_popup_result = self.approve_tx_popup.handle_event(
            (event, area, transmitter, shutdown_signal, shared_state),
            |_| {
                self.state = State::ApprovingTokens;
                Ok(())
            },
            |_| Ok(()),
            || Ok(()),
            || Ok(()),
        )?;
        result.merge(approve_tx_popup_result);

        let mut page_pop_on_cancel = false;
        let mut page_pop_on_esc = false;
        let sign_popup_result = self.sign_popup.handle_event(
            (event, area, transmitter, shared_state),
            |signature, popup| {
                let Some(prepared_order) = &self.prepared_order else {
                    return Err(crate::Error::InternalErrorStr("Prepared order is None"));
                };
                let Some(secret_hashes) = &self.secret_hashes else {
                    return Err(crate::Error::InternalErrorStr("secret_hashes is None"));
                };
                let Some(secrets) = &self.secrets else {
                    return Err(crate::Error::InternalErrorStr("secrets is None"));
                };

                self.state = State::SubmittingOrder;

                let relayer_request = RelayerRequest::from_prepared_order(
                    prepared_order,
                    signature,
                    self.quote_result
                        .as_ref()
                        .and_then(|q| q.quote_id.clone())
                        .ok_or(crate::Error::InternalErrorStr(
                            "No quote ID in quote result",
                        ))?,
                    if secret_hashes.len() == 1 {
                        None
                    } else {
                        Some(secret_hashes.clone())
                    },
                );

                self.submit_order_thread = Some(spawn_submit_order_thread(
                    transmitter.clone(),
                    relayer_request,
                    secrets.clone(),
                ));

                popup.close();
                Ok(())
            },
            || {
                page_pop_on_cancel = true;
                Ok(())
            },
            || {
                page_pop_on_esc = true;
                Ok(())
            },
        )?;
        result.merge(sign_popup_result);
        if page_pop_on_cancel || page_pop_on_esc {
            result.page_pops += 1;
        }

        Ok(result)
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
        [format!(
            "Transfer via Fusion Plus\n{} to {}\nCurrent Status: {}",
            self.src_chain.name, self.dst_chain.name, self.state
        )]
        .render(area, buf, false);

        self.display.render(area.margin_top(2), buf);

        self.quote_confirm_popup
            .render(area, buf, &shared_state.theme);

        self.approve_tx_popup.render(area, buf, &shared_state.theme);

        self.sign_popup.render(area, buf, &shared_state.theme);

        area
    }
}
