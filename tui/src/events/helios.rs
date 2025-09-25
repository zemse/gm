use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc, RwLock,
    },
    time::Duration,
};

use alloy::{
    eips::BlockId,
    primitives::{b256, TxKind},
    rpc::types::{TransactionInput, TransactionRequest},
};
use helios_ethereum::{
    config::networks::Network as HeliosNetwork, EthereumClient, EthereumClientBuilder,
};

use crate::Event;
use gm_utils::{
    assets::{AssetManager, LightClientVerification, TokenAddress},
    config::Config,
    erc20,
    network::Network,
};

pub async fn helios_thread(
    transmitter: &Sender<Event>,
    shutdown_signal: &Arc<AtomicBool>,
    asset_manager: Arc<RwLock<AssetManager>>,
) -> crate::Result<()> {
    let eth_network = Network::from_chain_id(1)?;

    let eth_client = EthereumClientBuilder::new()
        // Set the network to mainnet
        .network(HeliosNetwork::Mainnet)
        // Set the consensus rpc url
        // TODO handle situation when this website is down
        .consensus_rpc("https://www.lightclientdata.org")?
        // Set the execution rpc url
        .execution_rpc(eth_network.get_rpc()?)?
        // Set the checkpoint to the last known checkpoint
        .checkpoint(b256!(
            // https://beaconcha.in/slot/12256000
            "0x41a6280f0bdd34b8c4a2bc53fe934418cc600dc7cb7ede8fa7bc0e527557a1bc"
        ))
        // Set the data dir
        .data_dir(PathBuf::from("/tmp/helios"))
        // Set the fallback service
        .fallback("https://sync-mainnet.beaconcha.in")?
        // Enable lazy checkpoints
        .load_external_fallback()
        // Select the FileDB
        .with_file_db()
        .build()?;

    eth_client.wait_synced().await?;

    let mut dur = Duration::from_secs(0);
    loop {
        // wait for 10 seconds
        tokio::select! {
            result = run_interval(dur, transmitter, &asset_manager, &eth_client) => {
                if result.is_ok() {
                    dur = Duration::from_secs(10);
                } else {
                    dur += Duration::from_secs(30);
                }
            },
            _ = async {
                while !shutdown_signal.load(Ordering::Relaxed) {
                    tokio::task::yield_now().await;
                }
            } => break,
        }
    }

    eth_client.shutdown().await;

    Ok(())
}

async fn run_interval(
    wait_for: Duration,
    transmitter: &Sender<Event>,
    asset_manager: &Arc<RwLock<AssetManager>>,
    eth_client: &EthereumClient,
) -> crate::Result<()> {
    tokio::time::sleep(wait_for).await;

    let owner = Config::current_account()?;
    let assets = asset_manager
        .read()
        .map_err(|_| crate::Error::Poisoned("helios->run".to_string()))
        .and_then(|am| {
            am.get_assets(&owner)
                .cloned()
                .ok_or(crate::Error::AssetsNotFound(owner))
        })?;

    for asset in assets {
        let token_address = &asset.r#type.token_address;
        let expected_balance = asset.value;
        let network = Network::from_name(&asset.r#type.network)?;

        // TODO support other networks
        if network.chain_id != 1 {
            continue;
        }

        let actual_balance = match token_address {
            TokenAddress::Native => eth_client.get_balance(owner, BlockId::latest()).await?,
            TokenAddress::Contract(token_address) => {
                let result = eth_client
                    .call(
                        &TransactionRequest {
                            to: Some(TxKind::Call(*token_address)),
                            input: TransactionInput::from(erc20::encode_balance_of(owner)),
                            ..Default::default()
                        },
                        BlockId::latest(),
                    )
                    .await?;
                erc20::decode_balance_of(result)?
            }
        };

        let _ = transmitter.send(Event::HeliosUpdate {
            account: owner,
            network: asset.r#type.network.clone(),
            token_address: token_address.clone(),
            status: if actual_balance == expected_balance {
                LightClientVerification::Verified
            } else {
                LightClientVerification::Rejected
            },
        });
    }

    Ok(())
}
