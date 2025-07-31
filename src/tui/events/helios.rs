use std::{
    path::PathBuf,
    sync::{mpsc::Sender, Arc, RwLock},
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

use crate::{
    disk::Config,
    network::NetworkStore,
    tui::Event,
    utils::{
        assets::{AssetManager, LightClientVerification, TokenAddress},
        erc20,
    },
    Error,
};

pub async fn helios_thread(
    transmitter: &Sender<Event>,
    asset_manager: Arc<RwLock<AssetManager>>,
) -> crate::Result<()> {
    let eth_network = NetworkStore::from_chain_id(1)?;

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

    eth_client.wait_synced().await;

    loop {
        let _ = run(transmitter, &asset_manager, &eth_client).await;

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn run(
    transmitter: &Sender<Event>,
    asset_manager: &Arc<RwLock<AssetManager>>,
    eth_client: &EthereumClient,
) -> crate::Result<()> {
    let owner = Config::current_account()?.ok_or(Error::CurrentAccountNotSet)?;
    let assets = asset_manager
        .read()
        .map_err(|err| format!("{err}"))
        .and_then(|am| {
            am.get_assets(&owner)
                .cloned()
                .ok_or("no assets".to_string())
        })?;

    for asset in assets {
        let token_address = &asset.r#type.token_address;
        let expected_balance = asset.value;
        let network = NetworkStore::from_name(&asset.r#type.network)?;

        // TODO support other networks
        if network.chain_id != 1 {
            continue;
        }

        let actual_balance = match token_address {
            TokenAddress::Native => {
                eth_client
                    .get_balance(owner.as_raw(), BlockId::latest())
                    .await?
            }
            TokenAddress::Contract(token_address) => {
                let result = eth_client
                    .call(
                        &TransactionRequest {
                            to: Some(TxKind::Call((*token_address).as_raw())),
                            input: TransactionInput::from(erc20::encode_balance_of(owner.as_raw())),
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
