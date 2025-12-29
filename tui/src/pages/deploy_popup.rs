use std::path::Path;

use alloy::{
    hex,
    primitives::{Address, Bytes},
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
#[derive(Debug)]
pub enum DeployPopup {
    Closed,
    /// User needs to select a network
    SelectNetwork {
        bytecode: Bytes,
        contract_name: String,
        account: Address,
        networks_popup: Box<NetworksPopup>,
    },
    /// Network selected, now showing transaction popup
    Transaction { tx_popup: Box<SignTxPopup> },
}

impl Default for DeployPopup {
    fn default() -> Self {
        Self::Closed
    }
}

impl DeployPopup {
    /// Start deploy flow from a Foundry artifact JSON file
    pub fn from_artifact_path(
        path: &Path,
        network: Option<Network>,
        account: Address,
        networks: &NetworkStore,
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

        Ok(Self::start(bytecode, contract_name, network, account, networks))
    }

    /// Start deploy flow with bytecode directly
    pub fn start(
        bytecode: Bytes,
        contract_name: String,
        network: Option<Network>,
        account: Address,
        networks: &NetworkStore,
    ) -> Self {
        if let Some(network) = network {
            let tx_request = TransactionRequest {
                input: TransactionInput::new(bytecode),
                ..Default::default()
            };
            Self::Transaction {
                tx_popup: Box::new(SignTxPopup::new(account, network, tx_request)),
            }
        } else {
            let mut popup = networks_popup();
            popup.set_items(Some(networks.networks.clone()));
            popup.open();
            Self::SelectNetwork {
                bytecode,
                contract_name,
                account,
                networks_popup: Box::new(popup),
            }
        }
    }

    pub fn is_open(&self) -> bool {
        match self {
            DeployPopup::Closed => false,
            DeployPopup::SelectNetwork { networks_popup, .. } => networks_popup.is_open(),
            DeployPopup::Transaction { tx_popup } => tx_popup.is_open(),
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
                networks_popup,
            } => {
                if let Some(selected_network) =
                    networks_popup.handle_event(event.input_event(), popup_area, actions)?
                {
                    let network = (**selected_network).clone();
                    let tx_request = TransactionRequest {
                        input: TransactionInput::new(bytecode.clone()),
                        ..Default::default()
                    };
                    *self = DeployPopup::Transaction {
                        tx_popup: Box::new(SignTxPopup::new(*account, network, tx_request)),
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
            DeployPopup::Transaction { tx_popup } => {
                match tx_popup.handle_event(event, popup_area, actions)? {
                    Some(SignTxEvent::Cancelled) | Some(SignTxEvent::Done) => {
                        *self = DeployPopup::Closed;
                        Ok(Some(DeployEvent::Done))
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
            DeployPopup::Transaction { tx_popup } => {
                tx_popup.render(popup_area, buf, &shared_state.theme);
            }
        }
    }
}
