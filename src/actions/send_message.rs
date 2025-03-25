use std::future::IntoFuture;
use alloy::{
    consensus::{SignableTransaction, Transaction, TxEip1559, TxEnvelope},
    hex,
    network::TxSignerSync,
    primitives::{bytes::BytesMut, Address, Bytes, TxKind, U256},
    providers::{Provider, ProviderBuilder},
    rlp::Encodable,
};
use tokio::runtime::Runtime;
use crate::{actions::account::load_wallet, disk::Config, network::Network};

/// Sends a message from the currently selected wallet to a recipient wallet.
pub async fn send_message(to: String, msg: String, network: Option<Network>) {
    // Retrieve the current account dynamically
    let sender_account = Config::current_account();

    // Load wallet for the current account
    let wallet = load_wallet(sender_account).expect("❌ Failed to load wallet");

    // Validate recipient address
    if !to.starts_with("0x") || to.len() != 42 {
        eprintln!("❌ Error: Invalid Ethereum address format: {}", to);
        std::process::exit(1);
    }
    let to_address: Address = to.trim().parse().expect("❌ Failed to parse recipient Ethereum address");

    // Encode message as transaction calldata
    let calldata = Bytes::from(msg.into_bytes());

    // Ensure network is provided
    let network = match network {
        Some(n) => n,
        None => {
            eprintln!("❌ Error: Network must be specified.");
            std::process::exit(1);
        }
    };

    // Setup provider
    let rpc_url = network.get_rpc().parse().expect("❌ Invalid RPC URL");
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // Construct transaction
    let mut tx = TxEip1559::default();
    tx.to = TxKind::Call(to_address);
    tx.input = calldata;
    tx.value = U256::ZERO; // No ETH transfer, just a message

    // Fetch nonce asynchronously
    let nonce = provider
        .get_transaction_count(sender_account)
        .await
        .expect("❌ Failed to fetch nonce");
    tx.nonce = nonce;

    // Fetch chain ID
    let chain_id = provider.get_chain_id()
        .await
        .expect("❌ Failed to fetch chain ID");
    tx.chain_id = chain_id;

    tx.gas_limit = 51_000;

    // Estimate gas fees
    let fee_estimation = provider.estimate_eip1559_fees(None)
        .await
        .expect("❌ Gas fee estimation failed");
    tx.max_priority_fee_per_gas = fee_estimation.max_priority_fee_per_gas;
    tx.max_fee_per_gas = fee_estimation.max_fee_per_gas;

    // Sign transaction
    let signature = wallet.sign_transaction_sync(&mut tx).expect("❌ Signing error");
    let tx_signed = SignableTransaction::into_signed(tx, signature);

    // Encode transaction
    let mut out = BytesMut::new();
    let tx_typed = TxEnvelope::Eip1559(tx_signed);
    tx_typed.encode(&mut out);
    let out = &out[2..];

    // Submit transaction
    let result = provider.send_raw_transaction(out)
        .await
        .expect("❌ Transaction submission failed");

    let tx_hash = hex::encode_prefixed(result.tx_hash());
    println!(
        " Message sent! Transaction Hash: {}",
        network.get_tx_url(tx_hash.as_str()).unwrap_or(tx_hash)
    );

    // Wait for transaction confirmation
    let receipt = result.get_receipt()
        .await
        .expect("❌ Failed to get receipt");
    println!(
        "Confirmed in block {}",
        receipt.block_number.map(|n| n.to_string()).unwrap_or("unknown".to_string())
    );
}
