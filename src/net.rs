use futures::channel::oneshot;
use std::sync::OnceLock;

pub fn agent() -> ureq::Agent {
    static A: OnceLock<ureq::Agent> = OnceLock::new();
    A.get_or_init(|| ureq::AgentBuilder::new().build()).clone()
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
