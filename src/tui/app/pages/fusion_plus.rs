use std::{
    sync::{atomic::AtomicBool, mpsc::Sender, Arc},
    time::Duration,
};

use alloy::{
    primitives::{keccak256, utils::format_units, B256, U256, U64},
    signers::SignerSync,
};

use fusion_plus_sdk::{
    addresses::get_limit_order_contract_address,
    api::Api as FusionPlusApi,
    chain_id::ChainId,
    constants::UINT_256_MAX,
    cross_chain_order::{CrossChainOrderParams, Fee, PreparedOrder},
    hash_lock::HashLock,
    multichain_address::MultichainAddress,
    quote::{QuoteRequest, QuoteResult},
    relayer_request::RelayerRequest,
    utils::{alloy::ERC20, random::get_random_bytes32},
};
use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    disk::Config,
    gm_log,
    network::{Network, Token},
    tui::{
        app::{
            widgets::{confirm_popup::ConfirmPopup, tx_popup::TxPopup},
            SharedState,
        },
        traits::{Component, CustomRender, HandleResult},
        Event,
    },
    utils::{account::AccountManager, Provider},
};

fn spawn_quoter_thread(
    tr: Sender<Event>,
    quote_request: QuoteRequest,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        gm_log!("quote_request: {:#?}", quote_request);

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

fn spawn_allowance_thread(
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

fn spawn_submit_order_thread(tr: Sender<Event>, rr: RelayerRequest) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let Ok(oneinch_api_key) = Config::oneinch_api_key() else {
            let _ = tr.send(Event::FusionPlusError(
                "OneInch API key not found in config".to_string(),
            ));
            return;
        };
        let api = FusionPlusApi::new("https://api.1inch.dev/fusion-plus", oneinch_api_key);

        match api.submit_order(rr).await {
            Ok(_) => {
                let _ = tr.send(Event::FusionPlusOrderSubmitted);
            }
            Err(err) => {
                let _ = tr.send(Event::FusionPlusError(err.to_string()));
            }
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
        )
        .unwrap()
    };
    Ok((secret_hashes, secrets, hash_lock))
}

pub struct FusionPlusPage {
    pub src_chain: Network,
    pub src_token: Token,
    pub src_amount: U256,
    pub dst_chain: Network,
    pub dst_token: Token,
    pub dst_address: MultichainAddress,

    pub quote_request: QuoteRequest,
    pub quote_thread: Option<tokio::task::JoinHandle<()>>,
    pub quote_result: Option<QuoteResult>,
    pub quote_confirm_popup: ConfirmPopup,

    pub allowance_thread: Option<tokio::task::JoinHandle<()>>,
    pub allowance_result: Option<U256>,
    pub approve_tx_popup: TxPopup,

    pub secrets: Option<Vec<B256>>,
    pub prepared_order: Option<PreparedOrder>,
    pub submit_order_thread: Option<tokio::task::JoinHandle<()>>,
    pub order_submitted: bool,
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
        let quote_request = QuoteRequest::new(
            ChainId::from_u32(src_chain.chain_id),
            ChainId::from_u32(dst_chain.chain_id),
            src_token.contract_address,
            dst_token.contract_address,
            src_amount,
            true,
            MultichainAddress::ZERO,
        );

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
            prepared_order: None,
            submit_order_thread: None,
            order_submitted: false,
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
                self.quote_request.maker_address = *current_account;

                self.quote_thread = Some(spawn_quoter_thread(
                    transmitter.clone(),
                    self.quote_request.clone(),
                ));
            }
        }

        match event {
            Event::FusionPlusQuoteResult(quote_result) => {
                gm_log!("quote_result: {:#?}", quote_result);
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
                gm_log!("allowance received");
                gm_log!(
                    "src_chain_id: {src_chain_id}, self.src_chain.chain_id: {}",
                    self.src_chain.chain_id
                );
                gm_log!(
                    "src_token: {src_token}, self.src_token.contract_address: {}",
                    self.src_token.contract_address
                );
                if *src_chain_id as u32 == self.src_chain.chain_id
                    && src_token.as_raw() == self.src_token.contract_address.as_raw()
                {
                    gm_log!("allowance: {allowance}");
                    self.allowance_result = Some(*allowance);

                    if *allowance < self.src_amount {
                        gm_log!("allowance < src");
                        let src_erc20 =
                            ERC20::new(src_token.as_raw(), self.src_chain.get_provider()?);
                        let spender = get_limit_order_contract_address(*src_chain_id);
                        let call = src_erc20
                            .approve(spender.as_raw(), UINT_256_MAX)
                            .into_transaction_request();
                        gm_log!("self.src_chain {}", self.src_chain);
                        if !self.approve_tx_popup.is_open() {
                            self.approve_tx_popup
                                .set_tx_req(self.src_chain.clone(), call);
                            self.approve_tx_popup.open();
                        }
                    } else if let Some(quote_result) = &self.quote_result {
                        gm_log!("allowance > src");
                        let (secret_hashes, secrets, hash_lock) = prepare_secrets(quote_result)?;
                        self.secrets = Some(secrets);

                        let order = PreparedOrder::from_quote(
                            &self.quote_request,
                            quote_result,
                            CrossChainOrderParams {
                                dst_address: self.dst_address.without_chain_id(),
                                hash_lock,
                                secret_hashes: secret_hashes.clone(),
                                fee: Some(Fee {
                                    taking_fee_bps: 100,
                                    taking_fee_receiver: MultichainAddress::ZERO,
                                }),
                                preset: None,
                            },
                        )
                        .unwrap();
                        self.prepared_order = Some(order.clone());
                        gm_log!("prepared order: {:#?}", order);
                        let signer = AccountManager::load_wallet(
                            &shared_state.try_current_account()?.as_raw(),
                        )?;
                        let order_hash = order.eip712_signing_hash();
                        let signature = signer.sign_hash_sync(&order_hash)?;
                        let relayer_request = RelayerRequest::from_prepared_order(
                            &order,
                            signature,
                            quote_result.quote_id.clone().unwrap(),
                            if secret_hashes.len() == 1 {
                                None
                            } else {
                                Some(secret_hashes)
                            },
                        );
                        gm_log!("relayer_request: {:#?}", relayer_request);

                        self.submit_order_thread = Some(spawn_submit_order_thread(
                            transmitter.clone(),
                            relayer_request,
                        ));
                    }
                }
            }
            Event::FusionPlusOrderSubmitted => self.order_submitted = true,
            _ => {}
        }

        let confirm_popup_result = self.quote_confirm_popup.handle_event(
            event,
            area,
            || {
                gm_log!("allowance started, self.src_chain = {:#?}", self.src_chain);
                self.allowance_thread = Some(spawn_allowance_thread(
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
            |_| Ok(()),
            |_| Ok(()),
            || Ok(()),
            || Ok(()),
        )?;
        result.merge(approve_tx_popup_result);

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
        [
            "Fusion Plus Transfer",
            &format!("Source Chain: {}", self.src_chain),
            &format!("Source Token: {}", self.src_token),
            &format!("Source Amount: {}", self.src_amount),
            &format!("Destination Chain: {}", self.dst_chain),
            &format!("Destination Token: {}", self.dst_token),
            &format!("Destination Address: {}", self.dst_address),
            &format!("Order submited: {}", self.order_submitted),
        ]
        .render(area, buf, ());

        self.quote_confirm_popup
            .render(area, buf, &shared_state.theme);

        self.approve_tx_popup.render(area, buf, &shared_state.theme);

        area
    }
}
