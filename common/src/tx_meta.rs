use alloy::{
    consensus::TxEip1559,
    primitives::{utils::format_units, TxKind},
    sol_types::SolCall,
};

use crate::erc20::IERC20;

#[derive(Debug, Default)]
pub struct TransactionMeta {
    // Native metadata
    pub tx_dest_is_contract: Option<bool>,
    pub tx_dest_name: Option<String>,
    pub native_symbol: Option<String>,
    pub native_decimals: Option<u8>,

    // ERC20 metadata
    pub erc20_receiver_is_contract: Option<bool>,
    pub erc20_receiver_name: Option<String>,
    pub erc20_symbol: Option<String>,
    pub erc20_decimals: Option<u8>,

    // Optional function name if selector matches
    pub func_name: Option<String>,
}

impl TransactionMeta {
    /// Returns a user-friendly message describing the transaction.
    /// Apple Auth Prompt already has "gm wants to", so we provide the rest of text.
    pub fn get_display_message(&self, tx: &TxEip1559) -> String {
        match tx.to {
            TxKind::Create => {
                let input_len = tx.input.len();
                format!(
                    "deploy a new contract with {} bytes of creation code",
                    input_len
                )
            }
            TxKind::Call(address) => {
                let tx_dest = if let Some(name) = &self.tx_dest_name {
                    format!("{name} ({address:#})")
                } else {
                    format!("{address:#}")
                };

                if tx.input.is_empty() {
                    if tx.value.is_zero() {
                        if self.tx_dest_is_contract.unwrap_or(true) {
                            format!("call {tx_dest} with no data and no value")
                        } else {
                            format!("send an empty transaction to {address:#}")
                        }
                    } else if let Some((decimals, amount)) =
                        self.native_decimals.and_then(|decimals| {
                            format_units(tx.value, decimals)
                                .ok()
                                .map(|fmt| (decimals, fmt))
                        })
                    {
                        let amount = amount.trim_end_matches('0').trim_end_matches('.');

                        format!(
                            "send {amount} {symbol} (decimals {decimals}) to {tx_dest}",
                            symbol = self
                                .native_symbol
                                .clone()
                                .unwrap_or_else(|| "ETH".to_string())
                        )
                    } else {
                        format!("send {} wei to {}", tx.value, tx_dest)
                    }
                } else {
                    #[allow(clippy::collapsible_else_if)]
                    if let Ok(approve_call) = IERC20::approveCall::abi_decode_validate(&tx.input) {
                        let spender = if let Some(name) = &self.erc20_receiver_name {
                            format!("{name} ({:#})", approve_call.spender)
                        } else {
                            format!("{:#}", approve_call.spender)
                        };

                        if let Some((decimals, amount)) = self.erc20_decimals.and_then(|decimals| {
                            format_units(approve_call.amount, decimals)
                                .ok()
                                .map(|fmt| (decimals, fmt))
                        }) {
                            let amount = amount.trim_end_matches('0').trim_end_matches('.');
                            format!(
                                "allow {spender} to spend on your {amount} {} (decimals {decimals})",
                                self.erc20_symbol
                                    .clone()
                                    .unwrap_or_else(|| "coins".to_string())
                            )
                        } else {
                            format!(
                                "allow {spender} to spend {} wei of the token {}",
                                approve_call.amount, tx_dest
                            )
                        }
                    } else if let Ok(transfer_call) =
                        IERC20::transferCall::abi_decode_validate(&tx.input)
                    {
                        let receiver = if let Some(name) = &self.erc20_receiver_name {
                            format!("{name} ({:#})", transfer_call.to)
                        } else {
                            format!("{:#}", transfer_call.to)
                        };

                        if let Some((decimals, amount)) = self.erc20_decimals.and_then(|decimals| {
                            format_units(transfer_call.amount, decimals)
                                .ok()
                                .map(|fmt| (decimals, fmt))
                        }) {
                            let amount = amount.trim_end_matches('0').trim_end_matches('.');
                            format!(
                                "send {amount} {} (decimals {decimals}) to {receiver}",
                                self.erc20_symbol
                                    .clone()
                                    .unwrap_or_else(|| "coins".to_string())
                            )
                        } else {
                            format!(
                                "send {} of the token {} to {}",
                                transfer_call.amount, tx_dest, receiver
                            )
                        }
                    } else if let Some(function_name) = &self.func_name {
                        if function_name.chars().any(|c| c.is_uppercase()) {
                            format!("call {function_name} on {tx_dest}")
                        } else {
                            format!("{function_name} on {tx_dest}")
                        }
                    } else if !self.tx_dest_is_contract.unwrap_or(true) {
                        format!(
                            "send {} bytes of calldata to non-contract {tx_dest}",
                            tx.input.len()
                        )
                    } else {
                        format!(
                            "interact with {tx_dest} with {} bytes of calldata",
                            tx.input.len()
                        )
                    }
                }
            }
        }
    }
}
