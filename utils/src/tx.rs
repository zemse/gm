use alloy::{
    consensus::{TxEip1559, TxType},
    primitives::{Address, FixedBytes},
    providers::Provider,
    rpc::{json_rpc::ErrorPayload, types::TransactionRequest},
};

use crate::network::Network;

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
