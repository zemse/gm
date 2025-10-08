//! gm shell page
//!
//! This page provides a shell interface within the TUI application. It allows users to
//! execute shell commands with certain environment variables set which can be utilised.
//!
//! For e.g. `gm run --export-private-key ts-node script.ts` would allow the user to run
//! `ts-node script.ts` in the shell page and the environment variable `PRIVATE_KEY` will
//! be set to the private key of the current account. This allows users to avoid placing
//! secrets in .env files.
//!
//! Providing private key to a script can be dangerous. Hence, gm also exposes EIP-1193
//! compatible providers and programs can make RPC calls to it to sign transactions, it
//! would trigger sign box in the TUI application.
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, BufRead, BufReader, Write},
    process::{self, Command, Stdio},
    sync::mpsc::Sender,
    thread,
};

use alloy::{hex, primitives::Address, rpc::types::TransactionRequest};
use gm_ratatui_extra::{
    act::Act, extensions::ThemedWidget, input_box::InputBox, text_scroll::TextScroll,
};
use gm_rpc_proxy::{
    error::RpcProxyError,
    rpc_types::{ErrorObj, ResponsePayload},
};
use gm_utils::{disk_storage::DiskStorageInterface, network::NetworkStore};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyModifiers},
    layout::Rect,
};
use serde_json::{json, Value};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use walletconnect_sdk::utils::random_bytes32;

use crate::{
    app::SharedState,
    pages::{
        sign_popup::{SignPopup, SignPopupEvent},
        tx_popup::TxPopup,
    },
    post_handle_event::PostHandleEventActions,
    traits::Component,
    AppEvent,
};

#[derive(Debug)]
enum ShellLine {
    UserInput(String),
    StdOut(String),
    StdErr(String),
}

#[derive(Debug)]
pub struct UserRequest {
    params: UserRequestParams,
    reply_to: Option<oneshot::Sender<ResponsePayload<Value>>>,
}

#[derive(Debug)]
pub enum UserRequestParams {
    SendTransaction([Box<TransactionRequest>; 1]),
    SignMessage((String, Address)),
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ShellUpdate {
    StdOut(String),
    StdOut_Error(io::Error),

    StdErr(String),
    StdErr_Error(io::Error),

    Wait(process::ExitStatus),

    Kill_Error(io::Error),

    RpcProxyRequest(RefCell<Option<UserRequest>>),
    RpcProxyThreadCrashed(RefCell<Option<(RpcProxyError, String)>>),
}

#[derive(Debug)]
pub struct ShellPage {
    cmd_lines: Vec<ShellLine>,
    display: TextScroll,
    text_cursor: usize,
    env_vars: Option<HashMap<String, String>>,
    requests: Vec<UserRequest>,
    tx_popup: TxPopup,
    sign_popup: SignPopup,
    prevent_ctrlc_exit: bool,

    kill_signal: CancellationToken,
    stdin: Option<process::ChildStdin>,
    stdout_thread: Option<thread::JoinHandle<()>>,
    stderr_thread: Option<thread::JoinHandle<()>>,
    wait_thread: Option<thread::JoinHandle<()>>,

    exit_signal: CancellationToken,
    server_threads: Option<Vec<tokio::task::JoinHandle<()>>>,
}

impl Default for ShellPage {
    fn default() -> Self {
        let mut page = Self {
            cmd_lines: vec![
                ShellLine::StdOut("Welcome to gm shell".to_string()),
                ShellLine::UserInput(String::new()),
            ],
            display: TextScroll::default(),
            text_cursor: 0,
            env_vars: None,
            requests: vec![],
            tx_popup: TxPopup::default(),
            sign_popup: SignPopup::default(),
            prevent_ctrlc_exit: true,

            kill_signal: CancellationToken::new(),
            stdin: None,
            stdout_thread: None,
            stderr_thread: None,
            wait_thread: None,

            exit_signal: CancellationToken::new(),
            server_threads: None,
        };

        page.display.text = page.full_text();

        page
    }
}

impl ShellPage {
    pub fn get_user_input_mut(&mut self) -> Option<(&mut String, &mut usize)> {
        self.cmd_lines.last_mut().and_then(|cmd_line| {
            if let ShellLine::UserInput(input) = cmd_line {
                Some((input, &mut self.text_cursor))
            } else {
                None
            }
        })
    }

    fn full_text(&self) -> String {
        self.cmd_lines
            .iter()
            .map(|cmd_line| match cmd_line {
                ShellLine::UserInput(input) => format!("> {}", input),
                ShellLine::StdOut(output) => output.clone(),
                ShellLine::StdErr(error) => format!("[STDERR] {}", error),
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn create_server_threads(
        &mut self,
        tr: &Sender<AppEvent>,
        ss: &SharedState,
    ) -> crate::Result<()> {
        let secret = hex::encode(random_bytes32());
        let networks = NetworkStore::load()?.networks;

        let mut server_threads = vec![];
        let mut env_vars = HashMap::new();

        let mut port = 9393;
        for network in networks {
            let rpc_url = network.get_rpc()?.parse()?;
            let secret_clone = secret.clone();
            let tr = tr.clone();
            let port_actual = network.rpc_port.unwrap_or(port);
            let network_name = network.name.clone();
            let current_account = ss.try_current_account()?;
            let exit_signal = self.exit_signal.clone();
            server_threads.push(tokio::spawn(async move {
                let tr_clone = tr.clone();
                let result = gm_rpc_proxy::serve(
                    port_actual,
                    &secret_clone,
                    rpc_url,
                    exit_signal,
                    move |request| {
                        if request.method == "eth_accounts" {
                            // Synchronous immediate response
                            Ok(gm_rpc_proxy::OverrideResult::Sync(
                                ResponsePayload::Success(json!([current_account])),
                            ))
                        } else if request.method == "eth_sendTransction" {
                            let (oneshot_tr, oneshot_rv) =
                                oneshot::channel::<ResponsePayload<Value>>();

                            let _ = tr.send(AppEvent::ShellUpdate(ShellUpdate::RpcProxyRequest(
                                RefCell::new(Some(UserRequest {
                                    params: UserRequestParams::SendTransaction(
                                        serde_json::from_value(
                                            request
                                                .params
                                                .ok_or(RpcProxyError::RequestMissingParams)?,
                                        )
                                        .map_err(RpcProxyError::RequestParseFailed)?,
                                    ),
                                    reply_to: Some(oneshot_tr),
                                })),
                            )));

                            Ok(gm_rpc_proxy::OverrideResult::Async(oneshot_rv))
                        } else if request.method == "personal_sign" {
                            let (oneshot_tr, oneshot_rv) =
                                oneshot::channel::<ResponsePayload<Value>>();

                            let _ = tr.send(AppEvent::ShellUpdate(ShellUpdate::RpcProxyRequest(
                                RefCell::new(Some(UserRequest {
                                    params: UserRequestParams::SignMessage(
                                        serde_json::from_value(
                                            request
                                                .params
                                                .ok_or(RpcProxyError::RequestMissingParams)?,
                                        )
                                        .map_err(RpcProxyError::RequestParseFailed)?,
                                    ),
                                    reply_to: Some(oneshot_tr),
                                })),
                            )));

                            Ok(gm_rpc_proxy::OverrideResult::Async(oneshot_rv))
                        } else {
                            Ok(gm_rpc_proxy::OverrideResult::NoOverride)
                        }
                    },
                )
                .await;
                if let Err(e) = result {
                    let _ = tr_clone.send(AppEvent::ShellUpdate(
                        ShellUpdate::RpcProxyThreadCrashed(RefCell::new(Some((e, network_name)))),
                    ));
                }
            }));

            env_vars.insert(
                format!("{}_RPC_URL", network.name.to_uppercase().replace(' ', "_")),
                format!("http://localhost:{port_actual}/{secret}"),
            );

            port += 1;
        }

        self.server_threads = Some(server_threads);
        self.env_vars = Some(env_vars);

        Ok(())
    }

    fn exit_shell_threads_sync(&mut self) -> bool {
        let active = self.stdout_thread.is_some()
            || self.stderr_thread.is_some()
            || self.wait_thread.is_some()
            || self.stdin.is_some();

        self.kill_signal.cancel();

        if let Some(stdin) = self.stdin.take() {
            drop(stdin);
        }

        if let Some(stdout_thread) = self.stdout_thread.take() {
            stdout_thread.join().unwrap();
        }

        if let Some(stderr_thread) = self.stderr_thread.take() {
            stderr_thread.join().unwrap();
        }

        active
    }

    fn exit_server_sync(&mut self) {
        self.exit_signal.cancel();
        if let Some(server_threads) = self.server_threads.take() {
            for thread in server_threads {
                thread.abort();
            }
            self.server_threads = None;
        }
    }
}

impl Component for ShellPage {
    async fn exit_threads(&mut self) {
        self.exit_shell_threads_sync();
        self.exit_server_sync();
    }

    fn handle_event(
        &mut self,
        event: &AppEvent,
        area: Rect,
        _popup_area: Rect,
        tr: &Sender<AppEvent>,
        _: &CancellationToken,
        ss: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        let mut actions = PostHandleEventActions::default();

        if self.server_threads.is_none() {
            self.create_server_threads(tr, ss)?;
        }

        self.display.handle_event(event.key_event(), area);

        if self.prevent_ctrlc_exit {
            actions.ignore_ctrlc();
        }

        #[allow(clippy::single_match)]
        match event {
            AppEvent::Input(input_event) => {
                match input_event {
                    Event::Key(key_event) => {
                        let mut scroll_to_bottom = false;

                        if let Some((text_input, text_cursor)) = self.get_user_input_mut() {
                            // Keyboard handling for input
                            scroll_to_bottom = InputBox::handle_event(
                                Some(input_event),
                                area,
                                text_input,
                                text_cursor,
                            );

                            // Additional handling on top of InputBox
                            match key_event.code {
                                KeyCode::Char(c) => {
                                    if c == 'c' && key_event.modifiers == KeyModifiers::CONTROL {
                                        self.cmd_lines.push(ShellLine::StdOut(
                                            "Press ctrl+c again to exit".to_string(),
                                        ));
                                        self.cmd_lines.push(ShellLine::UserInput(String::new()));
                                        self.text_cursor = 0;
                                        self.prevent_ctrlc_exit = false;
                                    } else {
                                        self.prevent_ctrlc_exit = true;
                                    }
                                }
                                KeyCode::Enter => {
                                    scroll_to_bottom = true;

                                    let text_input = text_input.clone();
                                    if text_input.trim().is_empty() {
                                        self.cmd_lines.push(ShellLine::UserInput(String::new()));
                                        self.text_cursor = 0;
                                    } else {
                                        self.kill_signal.cancel();
                                        let mut child = Command::new("sh")
                                            .arg("-c")
                                            .arg(text_input)
                                            // .env("CLICOLOR", "1")
                                            // .env("CLICOLOR_FORCE", "1")
                                            .envs(
                                                self.env_vars
                                                    .as_ref()
                                                    .ok_or(crate::Error::ShellEnvVarsNotSet)?,
                                            )
                                            .stdin(Stdio::piped())
                                            .stdout(Stdio::piped())
                                            .stderr(Stdio::piped())
                                            .spawn()
                                            .map_err(crate::Error::SpawnFailed)?;

                                        self.stdin = child.stdin.take();
                                        let stdout = child
                                            .stdout
                                            .take()
                                            .ok_or(crate::Error::StdoutNotAvailable)?;
                                        let stderr = child
                                            .stderr
                                            .take()
                                            .ok_or(crate::Error::StderrNotAvailable)?;

                                        let tr_stdout = tr.clone();
                                        self.stdout_thread = Some(thread::spawn(move || {
                                            let reader = BufReader::new(stdout);
                                            for line in reader.lines() {
                                                match line {
                                                    Ok(s) => {
                                                        let _ =
                                                            tr_stdout.send(AppEvent::ShellUpdate(
                                                                ShellUpdate::StdOut(s),
                                                            ));
                                                    }
                                                    Err(e) => {
                                                        let _ =
                                                            tr_stdout.send(AppEvent::ShellUpdate(
                                                                ShellUpdate::StdOut_Error(e),
                                                            ));
                                                    }
                                                }
                                            }
                                        }));

                                        let tr_stderr = tr.clone();
                                        self.stderr_thread = Some(thread::spawn(move || {
                                            let reader = BufReader::new(stderr);
                                            for line in reader.lines() {
                                                match line {
                                                    Ok(s) => {
                                                        let _ =
                                                            tr_stderr.send(AppEvent::ShellUpdate(
                                                                ShellUpdate::StdErr(s),
                                                            ));
                                                    }
                                                    Err(e) => {
                                                        let _ =
                                                            tr_stderr.send(AppEvent::ShellUpdate(
                                                                ShellUpdate::StdErr_Error(e),
                                                            ));
                                                    }
                                                }
                                            }
                                        }));

                                        let tr_stderr = tr.clone();
                                        let kill_signal = self.kill_signal.clone();
                                        self.wait_thread = Some(thread::spawn(move || {
                                            while !kill_signal.is_cancelled() {
                                                if let Ok(status) = child.try_wait() {
                                                    match status {
                                                        Some(s) => {
                                                            let _ = tr_stderr.send(
                                                                AppEvent::ShellUpdate(
                                                                    ShellUpdate::Wait(s),
                                                                ),
                                                            );
                                                        }
                                                        None => {}
                                                    }
                                                    thread::sleep(
                                                        std::time::Duration::from_millis(100),
                                                    );
                                                }
                                            }
                                            match child.kill() {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    let _ = tr_stderr.send(AppEvent::ShellUpdate(
                                                        ShellUpdate::Kill_Error(e),
                                                    ));
                                                }
                                            }
                                        }));
                                    }
                                }
                                _ => {
                                    self.prevent_ctrlc_exit = true;
                                }
                            }
                        } else if let Some(stdin) = &mut self.stdin {
                            match key_event.code {
                                KeyCode::Char(c) => {
                                    if c == 'c' && key_event.modifiers == KeyModifiers::CONTROL {
                                        self.exit_shell_threads_sync();
                                        self.cmd_lines.push(ShellLine::UserInput(String::new()));
                                    } else {
                                        write!(stdin, "{c}")
                                            .map_err(crate::Error::StdinWriteFailed)?;
                                    }
                                }
                                KeyCode::Enter => {
                                    writeln!(stdin).map_err(crate::Error::StdinWriteFailed)?;
                                }
                                KeyCode::Backspace => {
                                    write!(stdin, "\x7f")
                                        .map_err(crate::Error::StdinWriteFailed)?;
                                }
                                _ => {}
                            }
                        }

                        self.display.text = self.full_text();
                        if scroll_to_bottom {
                            self.display
                                .scroll_to_bottom(area.width as usize, area.height as usize);
                        }
                    }
                    Event::Mouse(_mouse_event) => {
                        if let Some((text_input, text_cursor)) = self.get_user_input_mut() {
                            // Mouse handling for input
                            let _ = InputBox::handle_event(
                                Some(input_event),
                                area,
                                text_input,
                                text_cursor,
                            );
                        }
                    }
                    _ => {}
                }
            }
            AppEvent::ShellUpdate(update) => {
                match update {
                    ShellUpdate::StdOut(stdout) => {
                        self.cmd_lines.push(ShellLine::StdOut(stdout.clone()));
                    }
                    ShellUpdate::StdOut_Error(error) => {
                        return Err(crate::Error::StdoutReadFailed(format!("{error:?}")));
                    }
                    ShellUpdate::StdErr(stderr) => {
                        self.cmd_lines.push(ShellLine::StdErr(stderr.clone()));
                    }
                    ShellUpdate::StdErr_Error(error) => {
                        return Err(crate::Error::StderrReadFailed(format!("{error:?}")));
                    }
                    ShellUpdate::Wait(exit_status) => {
                        self.exit_shell_threads_sync();
                        self.cmd_lines.push(ShellLine::StdOut(format!(
                            "Process exited with {}",
                            exit_status
                        )));
                        self.cmd_lines.push(ShellLine::UserInput(String::new()));
                        self.text_cursor = 0;
                    }
                    ShellUpdate::Kill_Error(error) => {
                        return Err(crate::Error::ProcessKillFailed(format!("{error:?}")));
                    }
                    ShellUpdate::RpcProxyRequest(request) => {
                        if let Some(request) = request.take() {
                            self.requests.push(request);
                        }
                    }
                    ShellUpdate::RpcProxyThreadCrashed(data) => {
                        let (error, network_name) = data
                            .take()
                            .ok_or(crate::Error::ValueAlreadyTaken("RpcProxyThreadCrashed"))?;

                        return Err(crate::Error::RpcProxyThreadCrashed(error, network_name));
                    }
                }

                self.display.text = self.full_text();
                self.display
                    .scroll_to_bottom(area.width as usize, area.height as usize);
            }
            _ => {}
        }

        if self.sign_popup.is_open() {
            actions.ignore_esc();
        }

        if let Some(request) = self.requests.first_mut() {
            match &request.params {
                UserRequestParams::SendTransaction(transaction_request) => {
                    todo!("{transaction_request:?}")
                }
                UserRequestParams::SignMessage((msg, address)) => {
                    let current = ss.try_current_account()?;
                    if *address != current {
                        return Err(crate::Error::RequestAsksForDifferentAddress {
                            asked: *address,
                            current,
                        });
                    }

                    if !self.sign_popup.is_open() {
                        self.sign_popup.open();
                        self.sign_popup.set_msg_hex(msg);
                    }

                    // TODO sign should also take shutdown signal
                    self.sign_popup
                        .handle_event((event, area, tr, ss), |sign_event| {
                            // oneshot's sender is consumed here, cannot be used again
                            match sign_event {
                                SignPopupEvent::Signed(signature) => {
                                    let reply_to = request.reply_to.take().ok_or(
                                        crate::Error::ValueAlreadyTaken("UserRequest.reply_to"),
                                    )?;
                                    reply_to
                                        .send(ResponsePayload::Success(
                                            json!(signature.to_string()),
                                        ))
                                        .map_err(|_| crate::Error::OneshotSendFailed)?;
                                }
                                SignPopupEvent::Rejected | SignPopupEvent::EscapedBeforeSigning => {
                                    let reply_to = request.reply_to.take().ok_or(
                                        crate::Error::ValueAlreadyTaken("UserRequest.reply_to"),
                                    )?;
                                    reply_to
                                        .send(ResponsePayload::Error(ErrorObj::user_denied()))
                                        .map_err(|_| crate::Error::OneshotSendFailed)?;
                                }
                                SignPopupEvent::EscapedAfterSigning => {}
                            }

                            Ok(())
                        })?;

                    // If sign popup is closed during this moment, remove the request from the queue
                    if !self.sign_popup.is_open() {
                        self.requests.remove(0);
                    }
                }
            }
        }

        Ok(actions)
    }

    fn render_component(
        &self,
        area: Rect,
        popup_area: Rect,
        buf: &mut Buffer,
        ss: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        self.display.render(area, buf, &ss.theme);

        self.tx_popup.render(popup_area, buf, &ss.theme);
        self.sign_popup.render(popup_area, buf, &ss.theme);

        area
    }
}
