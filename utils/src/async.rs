use std::{future::Future, pin::Pin, time::Duration};

use tokio::{sync::oneshot, task::JoinHandle, time::MissedTickBehavior};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct AsyncOnce<T> {
    pub thread: JoinHandle<()>,
    pub receiver: oneshot::Receiver<T>,
    pub cancel_token: CancellationToken,
}

/// Starts an asynchronous job in a separate Tokio task that runs once.
/// The result of the job is sent back through a oneshot channel.
///
/// - `job`: A closure that returns a future representing the asynchronous job to be executed.
///
/// Returns a tuple containing:
/// - A `JoinHandle<()>` for the spawned Tokio task.
/// - A `oneshot::Receiver<R>` to receive the result of the job once it's satisfactory.
/// - A `CancellationToken` that can be used to signal the task to stop urgently.
pub fn async_once_thread<R, F, Fut>(job: F) -> AsyncOnce<R>
where
    R: Send + 'static,
    F: FnOnce() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
{
    let cancel_token = CancellationToken::new();
    let (tr, rc) = oneshot::channel();
    let cancel_token_clone = cancel_token.clone();

    let thread = tokio::spawn(async move {
        tokio::select! {
             _ = cancel_token_clone.cancelled() => (),
            result = job() => {
                let _ = tr.send(result);
            }
        };
    });

    AsyncOnce {
        thread,
        receiver: rc,
        cancel_token,
    }
}

/// Runs a job until the check_result function returns true, retrying at specified intervals.
/// The results of the job are sent back through a oneshot channel.
///
/// - `retry`: A `Duration` specifying the interval at which to retry the job.
/// - `job`: A closure that returns a future representing the asynchronous job to be executed.
/// - `should_retry`: A closure that takes a reference to the job's result and returns
///   boolean indicating whether the job should be retried.
///
/// Returns a tuple containing:
/// - A `JoinHandle<()>` for the spawned Tokio task.
/// - A `oneshot::Receiver<R>` to receive the result of the job once it's satisfactory.
/// - A `CancellationToken` that can be used to signal the task to stop urgently.
pub fn async_retry_thread<S, FS, R, F, F2>(
    retry: Duration,
    state: FS,
    job: F,
    should_retry: F2,
) -> AsyncOnce<R>
where
    S: Send + Sync + 'static,
    FS: FnOnce() -> S + Send + 'static,
    R: Send + 'static,
    for<'a> F: Fn(&'a S) -> BoxFut<'a, R> + Send + Sync + 'static,
    F2: Fn(&R) -> bool + Send + Sync + 'static,
{
    let shutdown_signal = CancellationToken::new();
    let (tr, rc) = oneshot::channel();
    let shutdown_signal_clone = shutdown_signal.clone();

    let thread = tokio::spawn(async move {
        inner_retry(retry, state, job, should_retry, tr, &shutdown_signal_clone).await;
    });

    AsyncOnce {
        thread,
        receiver: rc,
        cancel_token: shutdown_signal,
    }
}

type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub async fn inner_retry<S, FS, R, F, F2>(
    retry: Duration,
    state: FS,
    job: F,
    should_retry: F2,
    tr: tokio::sync::oneshot::Sender<R>,
    shutdown_signal: &tokio_util::sync::CancellationToken,
) where
    S: Send + Sync + 'static,
    FS: FnOnce() -> S + Send + 'static,
    R: Send + 'static,
    for<'a> F: Fn(&'a S) -> BoxFut<'a, R> + Send + Sync + 'static,
    F2: Fn(&R) -> bool + Send + Sync + 'static,
{
    let state = state();

    let mut interval = tokio::time::interval(retry);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = shutdown_signal.cancelled() => break,
            _ = interval.tick() => {},
        }

        tokio::select! {
            _ = shutdown_signal.cancelled() => break,
            result = job(&state) => {
                if !should_retry(&result) {
                    let _ = tr.send(result);
                    break;
                }
            }
        }
    }
}
