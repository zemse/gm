use std::{fmt::Display, future::IntoFuture};

use super::account::load_wallet;
use crate::{
    disk::{Config, DiskInterface},
    impl_inquire_selection,
    network::{Network, NetworkStore},
    utils::{Handle, Inquire},
};

use alloy::{
    consensus::{SignableTransaction, TxEip1559, TxEnvelope},
    hex,
    network::TxSignerSync,
    primitives::{bytes::BytesMut, Address, TxKind, U256},
    providers::{Provider, ProviderBuilder},
    rlp::Encodable,
};
use clap::{ArgAction, Subcommand};
use inquire::{Select, Text};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tokio::runtime::Runtime;

/// Transaction subcommands
///
/// List - `gm tx ls`
/// Create - `gm tx new`
#[derive(Subcommand, Display, Debug, EnumIter)]
#[allow(clippy::large_enum_variant)]
pub enum TransactionActions {
    #[command(alias = "new")]
    Create {
        #[arg(long, short)]
        network: Option<Network>,

        #[arg(long, short)]
        to: Option<Address>,

        #[arg(long, short)]
        value: Option<U256>,

        #[arg(long, short, action = ArgAction::SetTrue)]
        confirm: Option<bool>,
    },

    #[command(alias = "ls")]
    List,
}

impl_inquire_selection!(TransactionActions, ());

impl Handle for TransactionActions {
    fn handle(&self, _carry_on: ()) {
        match self {
            TransactionActions::Create {
                network,
                to,
                value,
                confirm,
            } => {
                let tx_input = if confirm.unwrap_or_default() {
                    TransactionCreateCarryOn {
                        network: network.clone(),
                        to: *to,
                        value: *value,
                    }
                } else {
                    TransactionCreateActionsVec::inquire(&TransactionCreateCarryOn {
                        network: network.clone(),
                        to: *to,
                        value: *value,
                    })
                    .expect("tx create args must be provided happen")
                    .into_obj()
                };

                // Implement transaction creation logic

                let network = tx_input.network.as_ref().expect("network must be provided");
                let rpc_url = network.get_rpc().parse().expect("error parsing URL");
                let provider = ProviderBuilder::new().on_http(rpc_url);

                let mut tx = TxEip1559::default();

                if let Some(to) = tx_input.to {
                    tx.to = TxKind::Call(to);
                } else {
                    tx.to = TxKind::Create;
                }

                if let Some(value) = tx_input.value {
                    tx.value = value;
                }

                let current_account = Config::current_account();
                let result = provider.get_transaction_count(current_account);

                // Create a Tokio runtime
                let rt = Runtime::new().expect("runtime failed");

                // // Block on the async function
                let nonce = rt.block_on(result.into_future()).expect("result");
                tx.nonce = nonce;

                tx.chain_id = 11155111;
                tx.gas_limit = 21_000;

                let fee_estimation = rt
                    .block_on(provider.estimate_eip1559_fees(None).into_future())
                    .expect("estimate fees failed");
                tx.max_priority_fee_per_gas = fee_estimation.max_priority_fee_per_gas;
                tx.max_fee_per_gas = insert_gm_mark(fee_estimation.max_fee_per_gas);

                let signer = load_wallet(current_account).expect("wallet issue");

                let signature = signer
                    .sign_transaction_sync(&mut tx)
                    .expect("signing error");
                let tx_signed = tx.into_signed(signature);

                let mut out = BytesMut::new();
                let tx_typed = TxEnvelope::Eip1559(tx_signed);
                tx_typed.encode(&mut out);
                let out = &out[2..];

                // TODO submit this to all RPCs available parallely
                let result = rt
                    .block_on(provider.send_raw_transaction(out).into_future())
                    .expect("submit failure");

                let tx_hash = hex::encode_prefixed(result.tx_hash());
                println!(
                    "tx is pending: {}",
                    network.get_tx_url(tx_hash.as_str()).unwrap_or(tx_hash)
                );
                let receipt = rt.block_on(result.get_receipt()).expect("wait failed");

                println!(
                    "Confirmed in block {}",
                    receipt
                        .block_number
                        .map(|n| n.to_string())
                        .unwrap_or("unknown".to_string())
                )
            }
            TransactionActions::List => {
                println!("Listing all transactions...");
                // Implement listing logic
            }
        }
    }
}

fn insert_gm_mark(gas_price: u128) -> u128 {
    let last_4_digits = gas_price % 10000;
    if last_4_digits != 0 {
        gas_price - last_4_digits + 9393
    } else {
        gas_price + 9393
    }
}

// TODO rename this from actions to options
#[derive(Subcommand, EnumIter, Clone)]
pub enum TransactionCreateActions {
    #[command(alias = "net")]
    Network { network: Option<Network> },

    #[command(alias = "to")]
    To { to: Option<Address> },

    #[command(alias = "val")]
    Value { value: Option<U256> },

    #[command(alias = "submit")]
    Confirm,
}

impl Display for TransactionCreateActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionCreateActions::Network { network } => {
                write!(
                    f,
                    "Network: {}",
                    network
                        .as_ref()
                        .map(|n| n.to_string())
                        .unwrap_or("<empty>".to_string())
                )
            }
            TransactionCreateActions::To { to } => {
                write!(
                    f,
                    "To: {}",
                    to.as_ref()
                        .map(|a| a.to_string())
                        .unwrap_or("<empty>".to_string())
                )
            }
            TransactionCreateActions::Value { value } => {
                write!(f, "Value: {}", value.unwrap_or(U256::ZERO))
            }
            TransactionCreateActions::Confirm => write!(f, "Confirm"),
        }
    }
}

#[derive(Clone)]
struct TransactionCreateActionsVec(Vec<TransactionCreateActions>);

#[derive(Debug)]
pub struct TransactionCreateCarryOn {
    pub network: Option<Network>,
    pub to: Option<Address>,
    pub value: Option<U256>,
}

impl From<&TransactionCreateCarryOn> for TransactionCreateActionsVec {
    fn from(value: &TransactionCreateCarryOn) -> Self {
        let mut options = vec![
            TransactionCreateActions::Network { network: None },
            TransactionCreateActions::To { to: None },
            TransactionCreateActions::Value { value: None },
            TransactionCreateActions::Confirm,
        ];

        if let Some(network) = &value.network {
            options[0] = TransactionCreateActions::Network {
                network: Some(network.clone()),
            };
        }
        if let Some(to) = value.to {
            options[1] = TransactionCreateActions::To { to: Some(to) };
        }
        if let Some(value) = value.value {
            options[2] = TransactionCreateActions::Value { value: Some(value) };
        }

        TransactionCreateActionsVec(options)
    }
}

impl TransactionCreateActionsVec {
    fn into_obj(self) -> TransactionCreateCarryOn {
        let mut network = None;
        let mut to = None;
        let mut value = None;

        for action in self.0 {
            match action {
                TransactionCreateActions::Network { network: n } => network = n,
                TransactionCreateActions::To { to: t } => to = t,
                TransactionCreateActions::Value { value: v } => value = v,
                _ => {}
            }
        }

        TransactionCreateCarryOn { network, to, value }
    }
}

impl Inquire<TransactionCreateCarryOn> for TransactionCreateActionsVec {
    fn inquire(carry_on: &TransactionCreateCarryOn) -> Option<TransactionCreateActionsVec> {
        let mut options = TransactionCreateActionsVec::from(carry_on);

        loop {
            let selected = inquire::Select::new("Edit tx parameter:", options.0.clone())
                .with_formatter(&|a| format!("{a}"))
                .prompt()
                .ok()?;
            match selected {
                TransactionCreateActions::Network { .. } => {
                    let network = Select::new("Select network", NetworkStore::load().networks)
                        .prompt()
                        .ok();
                    options.0[0] = TransactionCreateActions::Network { network }
                }
                TransactionCreateActions::To { to } => {
                    let to = Text::new("Enter address:")
                        .with_initial_value(&to.unwrap_or_default().to_string())
                        .prompt()
                        .ok()
                        .and_then(|a| a.parse().ok());

                    options.0[1] = TransactionCreateActions::To { to }
                }
                TransactionCreateActions::Value { value } => {
                    let value = Text::new("Enter value:")
                        .with_initial_value(&value.unwrap_or_default().to_string())
                        .prompt()
                        .ok()
                        .and_then(|v| v.parse().ok());

                    options.0[2] = TransactionCreateActions::Value { value }
                }
                TransactionCreateActions::Confirm => break,
            }
        }

        Some(options)
    }
}
