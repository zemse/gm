use alloy::{
    consensus::{TxEip1559, TxType},
    primitives::{Address, FixedBytes},
    providers::Provider,
    rpc::{json_rpc::ErrorPayload, types::TransactionRequest},
    sol_types::SolCall,
};
use gm_common::{erc20::IERC20, tx_meta::TransactionMeta};

use crate::{
    network::{Network, Token},
    sourcify::Sourcify,
};

/// Build EIP-1559 transaction by estimating gas, nonce, chain ID, and gas fees.
/// Returns the built `TxEip1559` on success, or the original `TransactionRequest` and an error on failure.
pub async fn build(
    sender_account: Address,
    network: Network,
    mut tx: TransactionRequest,
) -> crate::Result<TxEip1559> {
    let provider = network.get_provider()?;

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
    let estimate = if estimate_result.is_err()
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
    }?;

    let estimate_plus = estimate * 110 / 100; // TODO allow to configure gas limit
    if let Some(gas) = tx.gas {
        tx.gas = Some(std::cmp::max(gas, estimate_plus));
    } else {
        tx.gas = Some(estimate_plus);
    }

    tx.transaction_type = Some(2); // EIP-1559 transaction type

    let tx = tx
        .transaction_type(TxType::Eip1559.into())
        .build_typed_tx()
        .map_err(|_| crate::Error::TxTypeNotSpecified)?
        .eip1559()
        .ok_or(crate::Error::TxTypeIsNotEip1559)?
        .clone();

    Ok(tx)
}

pub async fn meta(
    network: Network,
    tx: TransactionRequest,
    mut meta: TransactionMeta,
    token: Option<Token>,
) -> crate::Result<TransactionMeta> {
    let provider = network.get_provider()?;

    if let Some(to) = tx.to.and_then(|to| to.into_to()) {
        if meta.tx_dest_is_contract.is_none() {
            meta.tx_dest_is_contract = Some(!provider.get_code_at(to).await?.is_empty());
        }

        if meta.tx_dest_name.is_none() {
            meta.tx_dest_name = Sourcify::fetch_contract_name(network.chain_id as u64, to)
                .await
                .ok()
                .flatten();
        }

        if meta.native_symbol.is_none() || meta.native_decimals.is_none() {
            meta.native_symbol = network.symbol.clone();
            meta.native_decimals = network.native_decimals;
        }

        if let Some(input) = tx.input.input() {
            if let Some(erc20_receiver) =
                if let Ok(decoded) = IERC20::approveCall::abi_decode_validate(input) {
                    Some(decoded.spender)
                } else if let Ok(decoded) = IERC20::transferCall::abi_decode_validate(input) {
                    Some(decoded.to)
                } else {
                    None
                }
            {
                if meta.erc20_receiver_is_contract.is_none() {
                    meta.erc20_receiver_is_contract =
                        Some(!provider.get_code_at(erc20_receiver).await?.is_empty());
                }

                if meta.erc20_receiver_name.is_none() {
                    meta.erc20_receiver_name =
                        Sourcify::fetch_contract_name(network.chain_id as u64, erc20_receiver)
                            .await
                            .ok()
                            .flatten();
                }

                if let Some(token) = token {
                    meta.erc20_symbol = Some(token.symbol.clone());
                    meta.erc20_decimals = Some(token.decimals);
                } else {
                    let contract = IERC20::new(to, provider);
                    meta.erc20_symbol = contract.symbol().call().await.ok();
                    meta.erc20_decimals = contract.decimals().call().await.ok();
                }
            }
        }
    }

    Ok(meta)
}

pub enum SendTxResult {
    Submitted(FixedBytes<32>),
    JsonRpcError(ErrorPayload),
    Error(crate::Error),
}

fn gm_stamp(gas_price: u128) -> u128 {
    let last_4_digits = gas_price % 10000;
    gas_price - last_4_digits + if last_4_digits > 9393 { 19393 } else { 9393 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gm_stamp() {
        assert_eq!(gm_stamp(0), 9393);
        assert_eq!(gm_stamp(1), 9393);
        assert_eq!(gm_stamp(100), 9393);
        assert_eq!(gm_stamp(456), 9393);

        assert_eq!(gm_stamp(9999), 19393);
        assert_eq!(gm_stamp(9998), 19393);
        assert_eq!(gm_stamp(9998), 19393);

        assert_eq!(gm_stamp(10000), 19393);
        assert_eq!(gm_stamp(10001), 19393);
        assert_eq!(gm_stamp(10002), 19393);
        assert_eq!(gm_stamp(10003), 19393);
        assert_eq!(gm_stamp(10004), 19393);

        assert_eq!(gm_stamp(19998), 29393);
        assert_eq!(gm_stamp(19999), 29393);

        assert_eq!(gm_stamp(1238999), 1239393);
        assert_eq!(gm_stamp(1239999), 1249393);
    }
}
