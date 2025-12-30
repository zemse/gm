use std::path::Path;

use alloy::{
    hex,
    primitives::{Address, Bytes, FixedBytes, TxKind},
    rpc::types::{TransactionInput, TransactionRequest},
};
use gm_ratatui_extra::{extensions::ThemedWidget, popup::PopupWidget, thematize::Thematize};
use gm_utils::network::{Network, NetworkStore};
use ratatui::{buffer::Buffer, layout::Rect};
use serde::Deserialize;

use crate::{
    app::SharedState,
    post_handle_event::PostHandleEventActions,
    widgets::{networks_popup, NetworksPopup},
    AppEvent,
};

use super::sign_tx_popup::{SignTxEvent, SignTxPopup};

/// Nick's Factory address for deterministic CREATE2 deployments
/// https://github.com/Arachnid/deterministic-deployment-proxy
fn deterministic_deployment_proxy() -> Address {
    "0x4e59b44847b379578588920cA78FbF26c0B4956C"
        .parse()
        .unwrap()
}

/// Foundry contract artifact structure
#[derive(Deserialize)]
struct ContractArtifact {
    bytecode: ArtifactBytecode,
}

#[derive(Deserialize)]
struct ArtifactBytecode {
    object: String,
}

/// Event returned by DeployPopup
#[derive(Debug)]
pub enum DeployEvent {
    /// User cancelled the deploy (pressed ESC on network selection)
    Cancelled,
    /// Deploy transaction completed (confirmed or failed)
    Done,
}

/// Popup for deploying a contract
/// Manages network selection and transaction signing internally
/// Supports deploying to multiple networks sequentially
#[derive(Debug)]
pub enum DeployPopup {
    Closed,
    /// User needs to select a network (when no networks provided via CLI)
    SelectNetwork {
        bytecode: Bytes,
        contract_name: String,
        account: Address,
        /// Salt for CREATE2 deployment (None = regular CREATE)
        salt: Option<FixedBytes<32>>,
        networks_popup: Box<NetworksPopup>,
    },
    /// Deploying to one or more networks
    Transaction {
        tx_popup: Box<SignTxPopup>,
        /// Bytecode for creating subsequent transactions
        bytecode: Bytes,
        /// Account for signing
        account: Address,
        /// Salt for CREATE2 deployment (None = regular CREATE)
        salt: Option<FixedBytes<32>>,
        /// Remaining networks to deploy to after current one completes
        pending_networks: Vec<Network>,
        /// Number of completed deployments
        completed_count: usize,
        /// Total number of networks to deploy to
        total_count: usize,
    },
}

impl Default for DeployPopup {
    fn default() -> Self {
        Self::Closed
    }
}

impl DeployPopup {
    /// Build transaction request for deployment
    /// If salt is provided, uses CREATE2 via Nick's Factory
    /// Otherwise, uses regular CREATE (contract creation tx)
    fn build_tx_request(bytecode: &Bytes, salt: Option<FixedBytes<32>>) -> TransactionRequest {
        match salt {
            Some(salt) => {
                // CREATE2: send to Nick's Factory with salt + bytecode as calldata
                let mut calldata = Vec::with_capacity(32 + bytecode.len());
                calldata.extend_from_slice(salt.as_slice());
                calldata.extend_from_slice(bytecode);
                TransactionRequest {
                    to: Some(TxKind::Call(deterministic_deployment_proxy())),
                    input: TransactionInput::new(Bytes::from(calldata)),
                    ..Default::default()
                }
            }
            None => {
                // Regular CREATE: contract creation tx
                TransactionRequest {
                    input: TransactionInput::new(bytecode.clone()),
                    ..Default::default()
                }
            }
        }
    }

    /// Start deploy flow from a Foundry artifact JSON file
    pub fn from_artifact_path(
        path: &Path,
        networks_to_deploy: Vec<Network>,
        account: Address,
        salt: Option<FixedBytes<32>>,
        network_store: &NetworkStore,
    ) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let artifact: ContractArtifact = serde_json::from_str(&content)?;

        let bytecode_hex = artifact.bytecode.object;
        let bytecode_hex = bytecode_hex.strip_prefix("0x").unwrap_or(&bytecode_hex);
        let bytecode = Bytes::from(hex::decode(bytecode_hex).map_err(crate::Error::FromHexError)?);

        let contract_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Contract")
            .to_string();

        Ok(Self::start(
            bytecode,
            contract_name,
            networks_to_deploy,
            account,
            salt,
            network_store,
        ))
    }

    /// Start deploy flow with bytecode directly
    /// If networks is empty, shows network picker
    /// If networks has one or more, deploys to each sequentially
    pub fn start(
        bytecode: Bytes,
        contract_name: String,
        mut networks: Vec<Network>,
        account: Address,
        salt: Option<FixedBytes<32>>,
        network_store: &NetworkStore,
    ) -> Self {
        if networks.is_empty() {
            // No networks specified, show picker
            let mut popup = networks_popup();
            popup.set_items(Some(network_store.networks.clone()));
            popup.open();
            Self::SelectNetwork {
                bytecode,
                contract_name,
                account,
                salt,
                networks_popup: Box::new(popup),
            }
        } else {
            // Deploy to the first network, queue the rest
            let total_count = networks.len();
            let network = networks.remove(0);
            let tx_request = Self::build_tx_request(&bytecode, salt);
            Self::Transaction {
                tx_popup: Box::new(SignTxPopup::new(account, network, tx_request)),
                bytecode,
                account,
                salt,
                pending_networks: networks,
                completed_count: 0,
                total_count,
            }
        }
    }

    /// Create a transaction popup for the next network
    fn start_next_network(&mut self) {
        if let DeployPopup::Transaction {
            bytecode,
            account,
            salt,
            pending_networks,
            completed_count,
            total_count,
            ..
        } = self
        {
            if !pending_networks.is_empty() {
                let network = pending_networks.remove(0);
                let tx_request = Self::build_tx_request(bytecode, *salt);
                *self = DeployPopup::Transaction {
                    tx_popup: Box::new(SignTxPopup::new(*account, network, tx_request)),
                    bytecode: bytecode.clone(),
                    account: *account,
                    salt: *salt,
                    pending_networks: std::mem::take(pending_networks),
                    completed_count: *completed_count + 1,
                    total_count: *total_count,
                };
            }
        }
    }

    pub fn is_open(&self) -> bool {
        match self {
            DeployPopup::Closed => false,
            DeployPopup::SelectNetwork { networks_popup, .. } => networks_popup.is_open(),
            DeployPopup::Transaction { tx_popup, .. } => tx_popup.is_open(),
        }
    }

    pub fn close(&mut self) {
        *self = DeployPopup::Closed;
    }

    pub fn handle_event(
        &mut self,
        event: &AppEvent,
        popup_area: Rect,
        actions: &mut PostHandleEventActions,
    ) -> crate::Result<Option<DeployEvent>> {
        match self {
            DeployPopup::Closed => Ok(None),
            DeployPopup::SelectNetwork {
                bytecode,
                contract_name: _,
                account,
                salt,
                networks_popup,
            } => {
                if let Some(selected_network) =
                    networks_popup.handle_event(event.input_event(), popup_area, actions)?
                {
                    let network = (**selected_network).clone();
                    let tx_request = Self::build_tx_request(bytecode, *salt);
                    *self = DeployPopup::Transaction {
                        tx_popup: Box::new(SignTxPopup::new(*account, network, tx_request)),
                        bytecode: bytecode.clone(),
                        account: *account,
                        salt: *salt,
                        pending_networks: vec![],
                        completed_count: 0,
                        total_count: 1,
                    };
                    Ok(None)
                } else if !networks_popup.is_open() {
                    // User closed the popup (pressed ESC)
                    *self = DeployPopup::Closed;
                    Ok(Some(DeployEvent::Cancelled))
                } else {
                    Ok(None)
                }
            }
            DeployPopup::Transaction {
                tx_popup,
                pending_networks,
                ..
            } => {
                match tx_popup.handle_event(event, popup_area, actions)? {
                    Some(SignTxEvent::Cancelled) => {
                        *self = DeployPopup::Closed;
                        Ok(Some(DeployEvent::Cancelled))
                    }
                    Some(SignTxEvent::Done) => {
                        if pending_networks.is_empty() {
                            // All deployments complete
                            *self = DeployPopup::Closed;
                            Ok(Some(DeployEvent::Done))
                        } else {
                            // Start next network deployment
                            self.start_next_network();
                            Ok(None)
                        }
                    }
                    _ => Ok(None),
                }
            }
        }
    }

    pub fn render(&self, popup_area: Rect, buf: &mut Buffer, shared_state: &SharedState) {
        match self {
            DeployPopup::Closed => {}
            DeployPopup::SelectNetwork { networks_popup, .. } => {
                networks_popup.render(popup_area, buf, &shared_state.theme.popup());
            }
            DeployPopup::Transaction { tx_popup, .. } => {
                tx_popup.render(popup_area, buf, &shared_state.theme);
            }
        }
    }
}
