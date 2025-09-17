use std::{
    io::{self, BufRead, BufReader, Write},
    process::{self, Command, Stdio},
    sync::{atomic::AtomicBool, mpsc::Sender, Arc},
    thread,
};

use gm_ratatui_extra::{input_box::InputBox, text_scroll::TextScroll};
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, layout::Rect, widgets::Widget};

use crate::{
    app::SharedState,
    traits::{Actions, Component},
    Event,
};

enum ShellLine {
    UserInput(String),
    StdOut(String),
    StdErr(String),
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ShellUpdate {
    StdOut(String),
    StdOut_Error(io::Error),

    StdErr(String),
    StdErr_Error(io::Error),

    Wait(process::ExitStatus),
    Wait_Error(io::Error),
}

pub struct ShellPage {
    cmd_lines: Vec<ShellLine>,
    display: TextScroll,
    text_cursor: usize,
    stdin: Option<process::ChildStdin>,
    stdout_thread: Option<thread::JoinHandle<()>>,
    stderr_thread: Option<thread::JoinHandle<()>>,
    wait_thread: Option<thread::JoinHandle<()>>,
}

impl Default for ShellPage {
    fn default() -> Self {
        Self {
            cmd_lines: vec![
                ShellLine::StdOut("Welcome to gm shell".to_string()),
                ShellLine::UserInput(String::new()),
            ],
            display: TextScroll::default(),
            text_cursor: 0,
            stdin: None,
            stdout_thread: None,
            stderr_thread: None,
            wait_thread: None,
        }
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

    fn exit_threads_sync(&mut self) {
        if let Some(stdin) = self.stdin.take() {
            drop(stdin);
        }
        if let Some(stdout_thread) = self.stdout_thread.take() {
            stdout_thread.join().unwrap();
        }
        if let Some(stderr_thread) = self.stderr_thread.take() {
            stderr_thread.join().unwrap();
        }
        if let Some(wait_thread) = self.wait_thread.take() {
            wait_thread.join().unwrap();
        }
    }
}

impl Component for ShellPage {
    async fn exit_threads(&mut self) {
        self.exit_threads_sync();
    }

    fn handle_event(
        &mut self,
        event: &Event,
        area: Rect,
        transmitter: &Sender<crate::Event>,
        _: &Arc<AtomicBool>,
        _: &SharedState,
    ) -> crate::Result<Actions> {
        self.display.handle_event(event.key_event(), area);

        #[allow(clippy::single_match)]
        match event {
            Event::Input(key_event) => {
                if let Some((text_input, text_cursor)) = self.get_user_input_mut() {
                    // Keyboard handling for input
                    InputBox::handle_event(Some(key_event), text_input, text_cursor);

                    // Execute
                    if key_event.code == KeyCode::Enter {
                        let text_input = text_input.clone();

                        if text_input.trim().is_empty() {
                            self.cmd_lines.push(ShellLine::UserInput(String::new()));
                            self.text_cursor = 0;
                        } else {
                            let parts: Vec<&str> = text_input.split_whitespace().collect();
                            let mut child = Command::new(parts[0])
                                .args(&parts[1..])
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

                            let tr_stdout = transmitter.clone();
                            self.stdout_thread = Some(thread::spawn(move || {
                                let reader = BufReader::new(stdout);
                                for line in reader.lines() {
                                    match line {
                                        Ok(s) => {
                                            let _ = tr_stdout
                                                .send(Event::ShellUpdate(ShellUpdate::StdOut(s)));
                                        }
                                        Err(e) => {
                                            let _ = tr_stdout.send(Event::ShellUpdate(
                                                ShellUpdate::StdOut_Error(e),
                                            ));
                                        }
                                    }
                                }
                            }));

                            let tr_stderr = transmitter.clone();
                            self.stderr_thread = Some(thread::spawn(move || {
                                let reader = BufReader::new(stderr);
                                for line in reader.lines() {
                                    match line {
                                        Ok(s) => {
                                            let _ = tr_stderr
                                                .send(Event::ShellUpdate(ShellUpdate::StdErr(s)));
                                        }
                                        Err(e) => {
                                            let _ = tr_stderr.send(Event::ShellUpdate(
                                                ShellUpdate::StdErr_Error(e),
                                            ));
                                        }
                                    }
                                }
                            }));

                            let tr_stderr = transmitter.clone();
                            self.wait_thread = Some(thread::spawn(move || {
                                let status = child.wait();
                                match status {
                                    Ok(s) => {
                                        let _ = tr_stderr
                                            .send(Event::ShellUpdate(ShellUpdate::Wait(s)));
                                    }
                                    Err(e) => {
                                        let _ = tr_stderr
                                            .send(Event::ShellUpdate(ShellUpdate::Wait_Error(e)));
                                    }
                                }
                            }));
                        }
                    }
                } else if let Some(stdin) = &mut self.stdin {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            write!(stdin, "{c}").map_err(crate::Error::StdinWriteFailed)?;
                        }
                        KeyCode::Enter => {
                            writeln!(stdin).map_err(crate::Error::StdinWriteFailed)?;
                        }
                        KeyCode::Backspace => {
                            write!(stdin, "\x7f").map_err(crate::Error::StdinWriteFailed)?;
                        }
                        _ => {}
                    }
                }
            }
            Event::ShellUpdate(update) => match update {
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
                    self.cmd_lines.push(ShellLine::StdOut(format!(
                        "Process exited with {}",
                        exit_status
                    )));

                    self.cmd_lines.push(ShellLine::UserInput(String::new()));
                    self.text_cursor = 0;

                    self.exit_threads_sync();
                }
                ShellUpdate::Wait_Error(error) => {
                    return Err(crate::Error::ProcessExitWaitFailed(format!("{error:?}")));
                }
            },
            _ => {}
        }

        self.display.text = self.full_text();
        self.display
            .scroll_to_bottom(area.width as usize, area.height as usize);

        Ok(Actions::default())
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, _: &SharedState) -> Rect
    where
        Self: Sized,
    {
        self.display.render(area, buf);

        area
    }
}
