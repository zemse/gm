use std::{sync::mpsc::Sender, time::Duration};

use tokio_util::sync::CancellationToken;

use crate::AppEvent;

pub async fn start_ticking(transmitter: Sender<AppEvent>, shutdown_signal: CancellationToken) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let _ = transmitter.send(AppEvent::Tick);
            }
            _ = shutdown_signal.cancelled() => break
        }
    }
}
