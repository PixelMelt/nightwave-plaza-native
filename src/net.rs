use futures::channel::oneshot;
use std::sync::LazyLock;
use std::time::Duration;

static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(15))
        .timeout_write(Duration::from_secs(10))
        .build()
});

pub fn agent() -> &'static ureq::Agent {
    &AGENT
}

pub fn read_body(
    result: Result<ureq::Response, ureq::Error>,
) -> Result<(Option<u16>, String), String> {
    let (status, resp) = match result {
        Ok(resp) => (None, resp),
        Err(ureq::Error::Status(code, resp)) => (Some(code), resp),
        Err(e) => return Err(e.to_string()),
    };
    Ok((status, resp.into_string().map_err(|e| e.to_string())?))
}

pub async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.await.expect("request thread panicked")
}
