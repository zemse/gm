use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        Arc,
    },
    time::Duration,
};

use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
};
use strum::EnumIter;
use tokio::task::JoinHandle;
use walletconnect_sdk::{
    pairing::{Pairing, Topic},
    types::{Metadata, SessionProposeParams},
    wc_message::{WcData, WcMessage},
    Connection,
};

use crate::tui::{
    app::{
        widgets::{
            confirm_popup::ConfirmPopup,
            form::{Form, FormItemIndex, FormWidget},
        },
        SharedState,
    },
    traits::Component,
    Event,
};

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

enum WcEvent {
    Message(Box<WcMessage>),
    NoOp,
}

#[derive(EnumIter, PartialEq)]
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
                label: "URI",
                text: String::new(),
                empty_text: Some("Paste Walletconnect URI from dapp"),
                currency: None,
            },
            FormItem::ConnectButton => FormWidget::Button { label: "Connect" },
        };
        Ok(widget)
    }
}

pub struct WalletConnectPage {
    form: Form<FormItem>,
    session_requests: Vec<WcMessage>,
    status: WalletConnectStatus,
    confirm_popup: ConfirmPopup,
    wait_popup: ConfirmPopup,
    watch_thread: Option<JoinHandle<()>>,
    send_thread: Option<JoinHandle<()>>,
    tr_2: Option<Sender<WcEvent>>,
}

impl WalletConnectPage {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            form: Form::init(|_| Ok(()))?,
            session_requests: vec![],
            status: WalletConnectStatus::Idle,
            confirm_popup: ConfirmPopup::new("WalletConnect", String::new(), "Approve", "Reject"),
            wait_popup: ConfirmPopup::new(
                "Warning",
                "The WalletConnect session will be ended if you go back. You can also press ESC to go back. If you want to continue session you can choose to wait."
                    .to_string(),
                "Wait",
                "End",
            ),
            watch_thread: None,
            send_thread: None,
            tr_2: None,
        })
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

    fn handle_event(
        &mut self,
        event: &Event,
        area: Rect,
        tr: &mpsc::Sender<Event>,
        sd: &Arc<AtomicBool>,
        shared_state: &SharedState,
    ) -> crate::Result<crate::tui::traits::HandleResult> {
        match event {
            Event::WalletConnectMessage(_addr, msg) => {
                match &msg.data {
                    WcData::SessionPing => {
                        if let Some(tr_2) = self.tr_2.as_ref() {
                            let _ = tr_2.send(WcEvent::Message(Box::new(
                                msg.create_response(WcData::SessionPingResponseSuccess),
                            )));
                        }
                    }
                    WcData::SessionRequest(_) => {
                        self.session_requests.push(*msg.clone());
                    }
                    _ => {
                        return Err(crate::Error::InternalError(format!(
                            "unhandled {:?} in TUI",
                            msg.method()
                        )))
                    }
                };
            }
            Event::WalletConnectStatus(status) => {
                self.status = status.clone();
                if let Some((_, proposal)) = status.proposal() {
                    let proposal = proposal.data.as_session_propose().ok_or(
                        crate::Error::InternalErrorStr("Not proposal, should not happen"),
                    )?;

                    let text = self.confirm_popup.text_mut();
                    *text = format_proposal(proposal);
                    self.confirm_popup.open();
                }
            }
            Event::WalletConnectError(_, _) => {
                self.status = WalletConnectStatus::Idle;
            }
            _ => {}
        }

        if self.status == WalletConnectStatus::Idle {
            self.form.handle_event(event, |item, form| {
                if item == FormItem::ConnectButton {
                    let uri_input = form.get_text(FormItem::UriInput).clone();
                    let current_account = shared_state.current_account.unwrap(); // TODO ensure we can see this page only if account exists
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
                        let _ = tr.send(Event::WalletConnectStatus(
                            WalletConnectStatus::Initializing,
                        ));

                        match conn.init_pairing(&uri_input).await {
                            Ok((pairing, proposal)) => {
                                let _ = tr.send(Event::WalletConnectStatus(
                                    WalletConnectStatus::ProposalReceived(Box::new((
                                        pairing, proposal,
                                    ))),
                                ));
                            }
                            Err(error) => {
                                let _ = tr.send(Event::WalletConnectError(
                                    current_account,
                                    format!("{error:?}"),
                                ));
                            }
                        };
                    });
                }
                Ok(())
            })?;
        }

        let mut handle_result = self.confirm_popup.handle_event(
            event,
            area,
            || {
                let pairing = self
                    .status
                    .proposal()
                    .ok_or(crate::Error::InternalErrorStr(
                        "proposal not found, cant happen",
                    ))?
                    .0
                    .clone();

                let _ = tr.send(Event::WalletConnectStatus(
                    WalletConnectStatus::SessionSettleInProgress,
                ));

                let addr = shared_state.current_account.unwrap();
                let tr = tr.clone();
                let shutdown_signal = sd.clone();
                let pairing_clone = pairing.clone();
                self.watch_thread = Some(tokio::spawn(async move {
                    let mut pairing = pairing_clone;
                    let Ok(msgs) = pairing.approve_with_session_settle(addr).await else {
                        let _ = tr.send(Event::WalletConnectStatus(
                            WalletConnectStatus::SessionSettleFailed,
                        ));
                        return;
                    };

                    let _ = tr.send(Event::WalletConnectStatus(
                        WalletConnectStatus::SessionSettleDone,
                    ));

                    for msg in msgs {
                        let _ = tr.send(Event::WalletConnectMessage(addr, Box::new(msg)));
                    }

                    loop {
                        if shutdown_signal.load(Ordering::Relaxed) {
                            break;
                        }

                        let messages = pairing.watch_messages(Topic::Derived, None).await.unwrap();

                        for msg in messages {
                            let _ = tr.send(Event::WalletConnectMessage(addr, Box::new(msg)));
                        }

                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }));

                let (tr_2, rc_2) = mpsc::channel::<WcEvent>();
                self.tr_2 = Some(tr_2);
                let shutdown_signal = sd.clone();
                self.send_thread = Some(tokio::spawn(async move {
                    loop {
                        if shutdown_signal.load(Ordering::Relaxed) {
                            break;
                        }

                        if let Ok(WcEvent::Message(msg)) = rc_2.recv() {
                            pairing
                                .send_message(
                                    Topic::Derived,
                                    &msg.into_raw().unwrap(), // TODO handle
                                    Some(0),
                                    msg.irn_tag(),
                                    msg.ttl(),
                                )
                                .await
                                .unwrap();
                        }
                    }
                }));

                Ok(())
            },
            || {
                let _ = tr.send(Event::WalletConnectStatus(
                    WalletConnectStatus::SessionSettleCancelled,
                ));
                Ok(())
            },
        )?;

        let mut go_back = false;

        let r = self.wait_popup.handle_event(
            event,
            area,
            || Ok(()),
            || {
                go_back = true;
                handle_result.page_pops += 1;
                Ok(())
            },
        )?;
        handle_result.merge(r);

        if let Event::Input(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    if let WalletConnectStatus::ProposalReceived(_) = self.status {
                        if !self.confirm_popup.is_open() && self.watch_thread.is_none() {
                            self.confirm_popup.open();
                        }
                    }
                }
                KeyCode::Esc => {
                    if self.status != WalletConnectStatus::Idle
                        // && !go_back
                        && (!self.confirm_popup.is_open() || !self.wait_popup.is_open())
                    {
                        self.wait_popup.open();
                    }
                }
                _ => {}
            }
        }

        if !go_back {
            handle_result.esc_ignores = 1;
        }

        Ok(handle_result)
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        shared_state: &crate::tui::app::SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        match &self.status {
            WalletConnectStatus::Idle => {
                self.form.render(area, buf, &shared_state.theme);
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

        self.confirm_popup.render(area, buf, &shared_state.theme);
        self.wait_popup.render(area, buf, &shared_state.theme);

        area
    }
}

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
