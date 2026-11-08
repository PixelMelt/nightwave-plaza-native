use futures::channel::oneshot;
use std::fmt;
use std::sync::LazyLock;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const UNAUTHORIZED: u16 = 401;

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

pub fn request(method: &str, url: &str) -> ureq::Request {
    AGENT.request(method, url).timeout(REQUEST_TIMEOUT)
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

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::http(UNAUTHORIZED, message)
    }

    pub fn other(message: impl fmt::Display) -> Self {
        Self {
            status: None,
            message: message.to_string(),
        }
    }

    pub fn transport(transport: &ureq::Transport) -> Self {
        let mut message = transport.kind().to_string();
        if let Some(detail) = transport.message() {
            message.push_str(": ");
            message.push_str(detail);
        }
        if let Some(source) = std::error::Error::source(transport) {
            message.push_str(": ");
            message.push_str(&source.to_string());
        }
        Self::other(message)
    }

    pub fn is_unauthorized(&self) -> bool {
        self.status == Some(UNAUTHORIZED)
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
        Err(ureq::Error::Transport(transport)) => return Err(Error::transport(&transport)),
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
