use std::{future::IntoFuture};

use alloy::{
    consensus::{TxEip1559, TxEnvelope},
    hex,
    network::TxSignerSync,
    primitives::{bytes::BytesMut, Address, TxKind, U256},
    providers::{Provider, ProviderBuilder},
    rlp::Encodable,
};
use tokio::runtime::Runtime;

use crate::{
    actions::account::load_wallet,
    disk::{Config, DiskInterface},
    network::Network,
    actions::sign_message::sign_message,
};

pub async fn send_message(to: String, msg: String, network: Option<Network>) {
    let current_account = Config::current_account();
    let wallet = load_wallet(current_account).expect("Failed to load wallet");

    let to_address: Address = to.parse().expect("Invalid Ethereum address");

    // Encode the message into calldata
    let calldata = msg.into_bytes();

    // Get the network RPC
    let network = network.unwrap_or_else(|| {
        panic!("Network must be specified")
    });

    let rpc_url = network.get_rpc().parse().expect("Invalid RPC URL");
    let provider = ProviderBuilder::new().on_http(rpc_url);

    let mut tx = TxEip1559::default();
    tx.to = TxKind::Call(to_address);
    tx.data = calldata.clone();
    tx.value = U256::ZERO;

    // Get nonce
    let result = provider.get_transaction_count(current_account);
    let rt = Runtime::new().expect("Failed to create runtime");
    let nonce = rt.block_on(result.into_future()).expect("Failed to fetch nonce");

    tx.nonce = nonce;
    tx.chain_id = 11155111; // Replace with correct chain ID
    tx.gas_limit = 21_000;

    // Estimate gas fees
    let fee_estimation = rt
        .block_on(provider.estimate_eip1559_fees(None).into_future())
        .expect("Gas fee estimation failed");

    tx.max_priority_fee_per_gas = fee_estimation.max_priority_fee_per_gas;
    tx.max_fee_per_gas = fee_estimation.max_fee_per_gas;

    // Sign the transaction
    let signature = wallet.sign_transaction_sync(&mut tx).expect("Signing error");
    let tx_signed = tx.into_signed(signature);

    let mut out = BytesMut::new();
    let tx_typed = TxEnvelope::Eip1559(tx_signed);
    tx_typed.encode(&mut out);
    let out = &out[2..];

    // Submit the transaction
    let result = rt
        .block_on(provider.send_raw_transaction(out).into_future())
        .expect("Transaction submission failed");

    let tx_hash = hex::encode_prefixed(result.tx_hash());
    println!(
        "✅ Message sent! Transaction Hash: {}",
        network.get_tx_url(tx_hash.as_str()).unwrap_or(tx_hash)
    );

    // Wait for confirmation
    let receipt = rt.block_on(result.get_receipt()).expect("Failed to get receipt");
    println!(
        "Confirmed in block {}",
        receipt
            .block_number
            .map(|n| n.to_string())
            .unwrap_or("unknown".to_string())
    );
}
