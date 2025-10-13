use std::{
    str::FromStr,
    sync::mpsc::{self, Sender},
    time::Duration,
};

use alloy::{hex, rpc::types::TransactionRequest};
use gm_ratatui_extra::{
    act::Act,
    button::Button,
    confirm_popup::{ConfirmPopup, ConfirmResult},
    cursor::Cursor,
    form::{Form, FormItemIndex, FormWidget},
    input_box_owned::InputBoxOwned,
    select::Select,
    thematize::Thematize,
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
};
use serde_json::Value;
use strum::{Display, EnumIter};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use walletconnect_sdk::{
    pairing::{Pairing, Topic},
    types::{IrnTag, Metadata, SessionProposeParams, SessionRequestData},
    wc_message::{WcData, WcMessage},
    Connection,
};

use crate::{
    app::SharedState,
    pages::{
        sign_popup::{SignPopup, SignPopupEvent},
        sign_typed_data_popup::SignTypedDataPopup,
        tx_popup::TxPopup,
    },
    post_handle_event::PostHandleEventActions,
    traits::Component,
    AppEvent,
};
use gm_utils::network::Network;

fn format_proposal(params: &SessionProposeParams) -> String {
    let metadata = &params.proposer.metadata;
    let mut output = format!(
        "dApp Name: {name}\n{desc}\n{url}\n\nRequested Permissions\n",
        name = metadata.name,
        desc = metadata.description,
        url = metadata.url
    );

    for (ns_key, ns) in params
        .required_namespaces
        .iter()
        .chain(params.optional_namespaces.iter())
    {
        output.push_str(&format!("\n1. {ns_key}\n"));

        if let Some(accounts) = &ns.accounts {
            output.push_str("   - Accounts:\n");
            for a in accounts {
                output.push_str(&format!("     • {a}\n"));
            }
        }

        if !ns.chains.is_empty() {
            output.push_str("   - Chains:\n");
            for c in &ns.chains {
                output.push_str(&format!("     • {c}\n"));
            }
        }

        if !ns.methods.is_empty() {
            output.push_str("   - Methods:\n");
            for m in &ns.methods {
                output.push_str(&format!("     • {m}\n"));
            }
        }

        if !ns.events.is_empty() {
            output.push_str("   - Events:\n");
            for e in &ns.events {
                output.push_str(&format!("     • {e}\n"));
            }
        }
    }

    output
}

#[derive(Clone, Debug, PartialEq)]
pub enum WalletConnectStatus {
    Idle,
    Initializing,
    ProposalReceived(Box<(Pairing, WcMessage)>),
    SessionSettleInProgress,
    SessionSettleDone,
    SessionSettleFailed,
    SessionSettleCancelled,
}

impl WalletConnectStatus {
    pub fn proposal(&self) -> Option<(&Pairing, &WcMessage)> {
        match self {
            WalletConnectStatus::ProposalReceived(boxxed) => {
                let (pairing, wc_message) = boxxed.as_ref();
                Some((pairing, wc_message))
            }
            _ => None,
        }
    }
}

pub enum WcEvent {
    Message(Box<WcMessage>),
    NoOp,
}

#[derive(Debug, Display, EnumIter, PartialEq)]
enum FormItem {
    Heading,
    UriInput,
    ConnectButton,
}

impl FormItemIndex for FormItem {
    fn index(self) -> usize {
        self as usize
    }
}

impl TryFrom<FormItem> for FormWidget {
    type Error = crate::Error;
    fn try_from(value: FormItem) -> crate::Result<Self> {
        let widget = match value {
            FormItem::Heading => FormWidget::Heading("Wallet Connect"),
            FormItem::UriInput => FormWidget::InputBox {
                widget: InputBoxOwned::new("URL")
                    .with_empty_text("Paste Walletconnect URI from dapp"),
            },
            FormItem::ConnectButton => FormWidget::Button {
                widget: Button::new("Connect"),
            },
        };
        Ok(widget)
    }
}

#[derive(Debug)]
pub struct WalletConnectPage {
    form: Form<FormItem, crate::Error>,
    session_requests: Vec<WcMessage>,
    cursor: Cursor,
    status: WalletConnectStatus,
    confirm_popup: ConfirmPopup,
    exit_popup: ConfirmPopup,
    tx_popup: TxPopup,
    sign_popup: SignPopup,
    sign_typed_data_popup: SignTypedDataPopup,
    watch_thread: Option<JoinHandle<()>>,
    send_thread: Option<JoinHandle<()>>,
    tr_2: Option<Sender<WcEvent>>,
}

impl WalletConnectPage {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            form: Form::init(|_| Ok(()))?,
            session_requests: vec![],
            cursor: Cursor::new(0),
            status: WalletConnectStatus::Idle,
            confirm_popup: ConfirmPopup::new("WalletConnect", String::new(), "Approve", "Reject", true),
            exit_popup: ConfirmPopup::new(
                "Warning",
                "The WalletConnect session will be ended if you go back. You can also press ESC to go back. If you want to continue session you can choose to wait."
                    .to_string(),
                "Wait",
                "End",
                false,
            ),
            tx_popup: TxPopup::default(),
            sign_popup: SignPopup::default(),
            sign_typed_data_popup: SignTypedDataPopup::default(),
            watch_thread: None,
            send_thread: None,
            tr_2: None,
        })
    }

    pub fn set_uri(&mut self, uri: &str) {
        self.form.set_text(FormItem::UriInput, uri.to_string());
    }

    fn open_request_at_cursor(&mut self) -> crate::Result<()> {
        let req = self.session_requests.get(self.cursor.current).ok_or(
            crate::Error::SessionRequestNotFound(self.cursor.current, self.session_requests.len()),
        )?;
        let req = req
            .data
            .as_session_request()
            .ok_or(crate::Error::NotSessionRequest)?;
        let chain_id = req
            .chain_id
            .strip_prefix("eip155:")
            .ok_or_else(|| crate::Error::ChainIdStripEip155Failed(req.chain_id.clone()))?
            .parse::<u32>()
            .map_err(|_| crate::Error::ChainIdParseFailed(req.chain_id.clone()))?;
        match &req.request.params {
            SessionRequestData::EthSendTransaction(tx_req) => {
                let network = Network::from_chain_id(chain_id)?;
                self.tx_popup.set_tx_req(
                    network,
                    TransactionRequest {
                        from: tx_req.from,
                        to: tx_req.to,
                        value: tx_req.value,
                        input: tx_req.input.clone(),
                        gas: tx_req.gas,
                        chain_id: tx_req.chain_id,
                        access_list: tx_req.access_list.clone(),
                        ..Default::default()
                    },
                );
                self.tx_popup.open();
            }
            SessionRequestData::PersonalSign { message, .. } => {
                self.sign_popup = SignPopup::new_with_message_hex(message)?;
                self.sign_popup.open();
            }
            SessionRequestData::EthSignTypedDataV4 {
                account: _,
                typed_data,
            } => {
                let mut typed_data = typed_data.clone();
                if let Some(str) = &typed_data.as_str() {
                    typed_data = Value::from_str(str)?;
                }
                self.sign_typed_data_popup.set_typed_data(typed_data)?;
                self.sign_typed_data_popup.open();
            }
        }

        Ok(())
    }
}

impl Component for WalletConnectPage {
    async fn exit_threads(&mut self) {
        let wc_thread = self.watch_thread.take();
        async move {
            if let Some(thread) = wc_thread {
                thread.abort();
                let _ = thread.await;
            }
        }
        .await;

        let send_thread = self.send_thread.take();
        async move {
            if let Some(thread) = send_thread {
                if let Some(tr_2) = self.tr_2.as_ref() {
                    let _ = tr_2.send(WcEvent::NoOp);
                }
                thread.abort();
                let _ = thread.await;
            }
        }
        .await;
    }

    fn set_focus(&mut self, focus: bool) {
        self.form.set_form_focus(focus);
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _popup_area: Rect,
        tr: &mpsc::Sender<AppEvent>,
        sd: &CancellationToken,
        ss: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let mut actions = PostHandleEventActions::default();

        let any_popup_open_before = self.confirm_popup.is_open()
            || self.exit_popup.is_open()
            || self.tx_popup.is_open()
            || self.sign_popup.is_open()
            || self.sign_typed_data_popup.is_open();

        // First handle the WalletConnect specific events regardless of what's there on the UI
        match event {
            AppEvent::WalletConnectMessage(_addr, msg) => {
                match &msg.data {
                    WcData::SessionPing => {
                        if let Some(tr_2) = self.tr_2.as_ref() {
                            let _ = tr_2.send(WcEvent::Message(Box::new(
                                msg.create_response(WcData::SessionPingResponseSuccess, None),
                            )));
                        }
                    }
                    WcData::SessionRequest(_) => {
                        self.session_requests.push(*msg.clone());
                        if !self.tx_popup.is_open() && !self.sign_popup.is_open() {
                            self.cursor.current = self.session_requests.len() - 1;
                            self.open_request_at_cursor()?;
                        }
                    }
                    // TODO handle session delete
                    _ => return Err(crate::Error::MethodUnhandled(msg.clone())),
                };
            }
            AppEvent::WalletConnectStatus(status) => {
                self.status = status.clone();
                if let Some((_, proposal)) = status.proposal() {
                    let proposal = proposal
                        .data
                        .as_session_propose()
                        .ok_or(crate::Error::ProposalNotFound)?;

                    let text = self.confirm_popup.text_mut();
                    *text = format_proposal(proposal);
                    self.confirm_popup.open();
                }
            }
            AppEvent::WalletConnectError(_, _) => {
                self.status = WalletConnectStatus::Idle;
            }
            _ => {}
        }

        let get_req_tr_2 = || -> crate::Result<_> {
            let req = self.session_requests.get(self.cursor.current).ok_or(
                crate::Error::SessionRequestNotFound(
                    self.cursor.current,
                    self.session_requests.len(),
                ),
            )?;
            let tr_2 = self
                .tr_2
                .as_ref()
                .ok_or(crate::Error::Transmitter2NotCreated)?;
            Ok((req, tr_2))
        };

        let mut go_back = false;
        let mut remove_current_request = false;
        let mut remove_current_request_2 = false;
        let mut remove_current_request_3 = false;

        // Handle based on what's active on the UI
        if self.confirm_popup.is_open() {
            match self
                .confirm_popup
                .handle_event(event.input_event(), area, &mut actions)?
            {
                Some(ConfirmResult::Confirmed) => {
                    let pairing = self
                        .status
                        .proposal()
                        .ok_or(crate::Error::ProposalNotFound)?
                        .0
                        .clone();

                    let _ = tr.send(AppEvent::WalletConnectStatus(
                        WalletConnectStatus::SessionSettleInProgress,
                    ));

                    {
                        let addr = ss.try_current_account()?;
                        let tr = tr.clone();
                        let shutdown_signal = sd.clone();
                        let pairing_clone = pairing.clone();
                        self.watch_thread = Some(tokio::spawn(async move {
                            let mut pairing = pairing_clone;
                            let Ok(msgs) = pairing.approve_with_session_settle(addr).await else {
                                let _ = tr.send(AppEvent::WalletConnectStatus(
                                    WalletConnectStatus::SessionSettleFailed,
                                ));
                                return;
                            };

                            let _ = tr.send(AppEvent::WalletConnectStatus(
                                WalletConnectStatus::SessionSettleDone,
                            ));

                            for msg in msgs {
                                let _ =
                                    tr.send(AppEvent::WalletConnectMessage(addr, Box::new(msg)));
                            }

                            while !shutdown_signal.is_cancelled() {
                                match pairing.watch_messages(Topic::Derived, None).await {
                                    Ok(messages) => {
                                        for msg in messages {
                                            let _ = tr.send(AppEvent::WalletConnectMessage(
                                                addr,
                                                Box::new(msg),
                                            ));
                                        }
                                    }
                                    Err(error) => {
                                        let _ = tr.send(AppEvent::WalletConnectError(
                                            addr,
                                            format!("Error during watch messages {error:?}"),
                                        ));
                                        break;
                                    }
                                }

                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }));
                    }

                    {
                        let (tr_2, rc_2) = mpsc::channel::<WcEvent>();
                        self.tr_2 = Some(tr_2);
                        let addr = ss.try_current_account()?;
                        let tr = tr.clone();
                        let shutdown_signal = sd.clone();
                        self.send_thread = Some(tokio::spawn(async move {
                            loop {
                                if shutdown_signal.is_cancelled() {
                                    break;
                                }

                                if let Ok(WcEvent::Message(msg)) = rc_2.recv() {
                                    match pairing
                                        .send_message(
                                            Topic::Derived,
                                            &msg.into_raw().unwrap(), // TODO handle unwrap
                                            Some(0),
                                            msg.irn_tag(),
                                            msg.ttl(),
                                        )
                                        .await
                                    {
                                        Ok(_) => {}
                                        Err(error) => tr
                                            .send(AppEvent::WalletConnectError(
                                                addr,
                                                format!("{error:?}"),
                                            ))
                                            .unwrap(),
                                    }
                                }
                            }
                        }));
                    }
                }
                Some(ConfirmResult::Canceled) => {
                    let _ = tr.send(AppEvent::WalletConnectStatus(
                        WalletConnectStatus::SessionSettleCancelled,
                    ));
                }
                None => {}
            }
        } else if self.tx_popup.is_open() {
            let r = self.tx_popup.handle_event(
                (event, area, tr, sd, ss),
                |tx_hash| {
                    let (req, tr_2) = get_req_tr_2()?;
                    tr_2.send(WcEvent::Message(Box::new(req.create_response(
                        WcData::SessionRequestResponse(Value::String(hex::encode_prefixed(
                            tx_hash,
                        ))),
                        None,
                    ))))?;
                    remove_current_request = true;
                    Ok(())
                },
                |_| Ok(()),
                |message, code, data| {
                    let (req, tr_2) = get_req_tr_2()?;
                    tr_2.send(WcEvent::Message(Box::new(req.create_response(
                        WcData::Error {
                            message,
                            code,
                            data,
                        },
                        Some(IrnTag::SessionRequestResponse),
                    ))))?;
                    remove_current_request_2 = true;

                    Ok(())
                },
                || {
                    let (req, tr_2) = get_req_tr_2()?;
                    tr_2.send(WcEvent::Message(Box::new(req.create_response(
                        WcData::Error {
                            message: "User denied tx signing".to_string(),
                            code: 5000,
                            data: None,
                        },
                        Some(IrnTag::SessionRequestResponse),
                    ))))?;
                    remove_current_request_3 = true;
                    Ok(())
                },
                || Ok(()),
            )?;
            actions.merge(r);
        } else if self.sign_popup.is_open() {
            if let Some(sign_popup_event) = self
                .sign_popup
                .handle_event((event, area, tr, ss), &mut actions)?
            {
                match sign_popup_event {
                    SignPopupEvent::Signed(_, signature) => {
                        let (req, tr_2) = get_req_tr_2()?;
                        tr_2.send(WcEvent::Message(Box::new(req.create_response(
                            WcData::SessionRequestResponse(Value::String(hex::encode_prefixed(
                                signature.as_bytes(),
                            ))),
                            None,
                        ))))?;
                        remove_current_request = true;
                    }
                    SignPopupEvent::Rejected => {
                        let (req, tr_2) = get_req_tr_2()?;
                        tr_2.send(WcEvent::Message(Box::new(req.create_response(
                            WcData::Error {
                                message: "User denied msg signing".to_string(),
                                code: 5000,
                                data: None,
                            },
                            Some(IrnTag::SessionRequestResponse),
                        ))))?;
                        remove_current_request_2 = true;
                    }
                    SignPopupEvent::EscapedBeforeSigning | SignPopupEvent::EscapedAfterSigning => {}
                }
            }
        } else if self.sign_typed_data_popup.is_open() {
            let r = self.sign_typed_data_popup.handle_event(
                (event, area, tr, ss),
                |signature| {
                    let (req, tr_2) = get_req_tr_2()?;
                    tr_2.send(WcEvent::Message(Box::new(req.create_response(
                        WcData::SessionRequestResponse(Value::String(hex::encode_prefixed(
                            signature.as_bytes(),
                        ))),
                        None,
                    ))))?;
                    remove_current_request = true;
                    Ok(())
                },
                || {
                    let (req, tr_2) = get_req_tr_2()?;
                    tr_2.send(WcEvent::Message(Box::new(req.create_response(
                        WcData::Error {
                            message: "User denied msg signing".to_string(),
                            code: 5000,
                            data: None,
                        },
                        Some(IrnTag::SessionRequestResponse),
                    ))))?;
                    remove_current_request_2 = true;
                    Ok(())
                },
                || Ok(()),
            )?;
            actions.merge(r);
        } else if self.exit_popup.is_open() {
            if let Some(ConfirmResult::Canceled) =
                self.exit_popup
                    .handle_event(event.input_event(), area, &mut actions)?
            {
                go_back = true;
                actions.page_pop();
            }
        } else if self.status == WalletConnectStatus::Idle {
            let r = self.form.handle_event(
                event.widget_event().as_ref(),
                area,
                |_, _| Ok(()),
                |item, form| {
                    if item == FormItem::ConnectButton {
                        let uri_input = form.get_text(FormItem::UriInput).to_string();
                        let current_account = ss.current_account.unwrap(); // TODO ensure we can see this page only if account exists
                        let tr = tr.clone();

                        let client_seed = [123u8; 32];
                        let project_id: &str = "46c07e56a92e34fe567dcc951fba3f3e";

                        let conn = Connection::new(
                            "https://relay.walletconnect.org/rpc",
                            "https://relay.walletconnect.org",
                            // TODO take project ID and client seed from config
                            project_id,
                            client_seed,
                            Metadata {
                                name: "gm wallet".to_string(),
                                description: "gm is a TUI based ethereum wallet".to_string(),
                                url: "https://github.com/zemse/gm".to_string(),
                                icons: vec![],
                            },
                        );

                        tokio::spawn(async move {
                            let _ = tr.send(AppEvent::WalletConnectStatus(
                                WalletConnectStatus::Initializing,
                            ));

                            match conn.init_pairing(&uri_input).await {
                                Ok((pairing, proposal)) => {
                                    let _ = tr.send(AppEvent::WalletConnectStatus(
                                        WalletConnectStatus::ProposalReceived(Box::new((
                                            pairing, proposal,
                                        ))),
                                    ));
                                }
                                Err(error) => {
                                    let _ = tr.send(AppEvent::WalletConnectError(
                                        current_account,
                                        format!("{error:?}"),
                                    ));
                                }
                            };
                        });
                    }
                    Ok(())
                },
            )?;
            actions.merge(r);
        } else if self.status == WalletConnectStatus::SessionSettleDone {
            self.cursor
                .handle(event.key_event(), self.session_requests.len());

            if let AppEvent::Input(input_event) = event {
                match input_event {
                    Event::Key(key_event) => {
                        if key_event.code == KeyCode::Enter {
                            self.open_request_at_cursor()?;
                        }
                    }
                    Event::Mouse(_mouse_event) => {}
                    _ => {}
                }
            }
        }

        if remove_current_request || remove_current_request_2 || remove_current_request_3 {
            self.session_requests.remove(self.cursor.current);
        }

        // Special handling for ESC key, Ask user if they really want to exit
        if let AppEvent::Input(input_event) = event {
            match input_event {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Esc
                        && self.status != WalletConnectStatus::Idle
                        && !self.confirm_popup.is_open()
                        && !self.exit_popup.is_open()
                        && !self.tx_popup.is_open()
                        && !self.sign_popup.is_open()
                        && !any_popup_open_before
                    {
                        self.exit_popup.open();
                    }
                }
                Event::Mouse(_mouse_event) => {}
                _ => {}
            }
        }

        if !go_back && self.status != WalletConnectStatus::Idle {
            actions.ignore_esc();
        }

        Ok(actions)
    }

    fn render_component(
        &self,
        area: Rect,
        popup_area: Rect,
        buf: &mut Buffer,
        shared_state: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        match &self.status {
            WalletConnectStatus::Idle => {
                self.form.render(area, popup_area, buf, &shared_state.theme);
            }
            WalletConnectStatus::Initializing => {
                "Initializing connection...".render(area, buf);
            }
            WalletConnectStatus::ProposalReceived(_) => {
                "Please confirm pairing details using the popup".render(area, buf);
            }
            WalletConnectStatus::SessionSettleInProgress => {
                "Settling session...".render(area, buf);
            }
            WalletConnectStatus::SessionSettleFailed => {
                "Settling session failed".render(area, buf);
            }
            WalletConnectStatus::SessionSettleCancelled => {
                "Settling session cancelled".render(area, buf);
            }
            WalletConnectStatus::SessionSettleDone => {
                if self.session_requests.is_empty() {
                    "Connected! Waiting for session requests".render(area, buf);
                } else {
                    Select {
                        focus: true,
                        list: &self
                            .session_requests
                            .iter()
                            .map(|r| format!("{r:?}"))
                            .collect::<Vec<_>>(),
                        cursor: &self.cursor,
                    }
                    .render(area, buf, None, &shared_state.theme);

                    let mut s = String::new();
                    for msg in &self.session_requests {
                        s.push_str(&format!("{msg:?}\n"));
                    }
                    Paragraph::new(Text::raw(&s))
                        .wrap(Wrap { trim: false })
                        .to_owned()
                        .render(area, buf);
                }
            }
        }

        self.confirm_popup
            .render(popup_area, buf, &shared_state.theme.popup());
        self.tx_popup
            .render(popup_area, buf, &shared_state.theme.popup());
        self.sign_popup
            .render(popup_area, buf, &shared_state.theme.popup());
        self.sign_typed_data_popup
            .render(popup_area, buf, &shared_state.theme.popup());
        self.exit_popup
            .render(popup_area, buf, &shared_state.theme.popup());

        area
    }
}
