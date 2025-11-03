use tokio_util::sync::CancellationToken;

pub async fn handle_abort<F, Fut>(
    shutdown_signal: &CancellationToken,
    f: F,
) -> crate::Result<Fut::Output>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future,
{
    tokio::select! {
        _ = shutdown_signal.cancelled() => {
            Err(crate::Error::AbortDueToShutdown)
        }
        result = f() => {
            Ok(result)
        }
    }
}
