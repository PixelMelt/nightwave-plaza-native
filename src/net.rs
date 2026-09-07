use futures::channel::oneshot;
use std::sync::OnceLock;
use std::time::Duration;

/// Shared HTTP agent. The read timeout is what lets a dead connection
/// (network drop, sleep/resume) surface as an error instead of blocking
/// a request or the radio stream forever.
pub fn agent() -> ureq::Agent {
    static A: OnceLock<ureq::Agent> = OnceLock::new();
    A.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(15))
            .timeout_write(Duration::from_secs(10))
            .build()
    })
    .clone()
}

pub fn read_body(
    result: Result<ureq::Response, ureq::Error>,
) -> Result<(Option<u16>, String), String> {
    match result {
        Ok(resp) => Ok((None, resp.into_string().map_err(|e| e.to_string())?)),
        Err(ureq::Error::Status(code, resp)) => {
            Ok((Some(code), resp.into_string().map_err(|e| e.to_string())?))
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.await {
        Ok(result) => result,
        Err(_) => Err("request thread terminated unexpectedly".into()),
    }
}
