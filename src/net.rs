use futures::channel::oneshot;
use std::fmt;
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

#[derive(Debug, Clone)]
pub struct Error {
    status: Option<u16>,
    message: String,
}

impl Error {
    pub fn http(status: u16, message: impl Into<String>) -> Self {
        Self {
            status: Some(status),
            message: message.into(),
        }
    }

    pub fn other(message: impl fmt::Display) -> Self {
        Self {
            status: None,
            message: message.to_string(),
        }
    }

    pub fn is_unauthorized(&self) -> bool {
        matches!(self.status, Some(401 | 403))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

pub struct Body {
    pub status: u16,
    pub text: String,
}

impl Body {
    pub fn is_error(&self) -> bool {
        self.status >= 400
    }
}

pub fn read_body(result: Result<ureq::Response, ureq::Error>) -> Result<Body, Error> {
    let response = match result {
        Ok(response) | Err(ureq::Error::Status(_, response)) => response,
        Err(e) => return Err(Error::other(e)),
    };
    let status = response.status();
    let text = response.into_string().map_err(Error::other)?;
    Ok(Body { status, text })
}

pub async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.await.expect("request thread panicked")
}
