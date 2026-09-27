use crate::api::StatusSong;
use crate::net::{self, Error, blocking};
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::time::Instant;

const API_KEY: &str = "e08418a04fb411affc437a8aab96cb89";
const API_SECRET: &str = "91a2c65df39ea28a7f305f7b35242e17";

const API_ROOT: &str = "https://ws.audioscrobbler.com/2.0/";
const AUTH_URL: &str = "https://www.last.fm/api/auth/";

const MIN_TRACK_SECS: f64 = 30.0;
const MAX_THRESHOLD_SECS: f64 = 240.0;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LastfmConfig {
    pub enabled: bool,
    pub session_key: Option<String>,
    pub username: Option<String>,
}

impl LastfmConfig {
    pub fn active_session_key(&self) -> Option<&str> {
        self.session_key.as_deref().filter(|_| self.enabled)
    }

    pub fn connected_username(&self) -> Option<&str> {
        self.session_key.as_ref().and(self.username.as_deref())
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub username: String,
    pub key: String,
}

pub struct Scrobble {
    pub song: StatusSong,
    started_at_unix: Option<u64>,
    played_secs: f64,
    playing_since: Option<Instant>,
}

impl Scrobble {
    pub fn new(song: StatusSong) -> Self {
        Self {
            song,
            started_at_unix: None,
            played_secs: 0.0,
            playing_since: None,
        }
    }

    pub fn set_playing(&mut self, playing: bool, now: Instant) {
        if playing {
            self.started_at_unix
                .get_or_insert_with(|| Utc::now().timestamp().unsigned_abs());
            self.playing_since.get_or_insert(now);
        } else if let Some(since) = self.playing_since.take() {
            self.played_secs += now.duration_since(since).as_secs_f64();
        }
    }

    pub fn due(&self, now: Instant) -> Option<u64> {
        let started_at = self.started_at_unix?;
        if !self.song.has_metadata() {
            return None;
        }
        let length = self.song.length;
        if length > 0.0 && length < MIN_TRACK_SECS {
            return None;
        }
        let threshold = if length > 0.0 {
            (length / 2.0).min(MAX_THRESHOLD_SECS)
        } else {
            MIN_TRACK_SECS
        };
        let current_stretch = self
            .playing_since
            .map_or(0.0, |since| now.duration_since(since).as_secs_f64());
        (self.played_secs + current_stretch >= threshold).then_some(started_at)
    }
}

fn sign(params: &[(&str, &str)]) -> String {
    let mut sorted: Vec<_> = params.iter().filter(|(k, _)| *k != "format").collect();
    sorted.sort_by_key(|(k, _)| *k);
    let mut buf: String = sorted.into_iter().flat_map(|(k, v)| [*k, *v]).collect();
    buf.push_str(API_SECRET);
    format!("{:x}", md5::compute(buf))
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(default)]
    error: i32,
    #[serde(default)]
    message: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Deserialize)]
struct SessionResponse {
    session: SessionBody,
}

#[derive(Deserialize)]
struct SessionBody {
    name: String,
    key: String,
}

fn parse<T: DeserializeOwned>(result: Result<ureq::Response, ureq::Error>) -> Result<T, Error> {
    let body = net::read_body(result)?;
    if let Ok(err) = serde_json::from_str::<ErrorBody>(&body.text)
        && err.error != 0
    {
        let message = if err.message.is_empty() {
            format!("Last.fm error {}", err.error)
        } else {
            err.message
        };
        return Err(Error::http(body.status, message));
    }
    serde_json::from_str(&body.text).map_err(Error::other)
}

fn get_signed<T: DeserializeOwned>(params: &[(&str, &str)]) -> Result<T, Error> {
    let sig = sign(params);
    let request = params
        .iter()
        .fold(net::agent().get(API_ROOT), |req, (k, v)| req.query(k, v));
    parse(
        request
            .query("api_sig", &sig)
            .query("format", "json")
            .call(),
    )
}

fn post_signed(params: Vec<(&str, &str)>) -> Result<(), Error> {
    let sig = sign(&params);
    let mut form: Vec<(&str, &str)> = params;
    form.push(("api_sig", &sig));
    form.push(("format", "json"));
    parse::<serde_json::Value>(net::agent().post(API_ROOT).send_form(&form))?;
    Ok(())
}

fn track_params<'a>(method: &'a str, sk: &'a str, song: &'a StatusSong) -> Vec<(&'a str, &'a str)> {
    let mut params = vec![
        ("method", method),
        ("api_key", API_KEY),
        ("sk", sk),
        ("artist", song.artist.as_str()),
        ("track", song.title.as_str()),
    ];
    if !song.album.is_empty() {
        params.push(("album", song.album.as_str()));
    }
    params
}

pub async fn fetch_token() -> Result<String, Error> {
    blocking(|| {
        let response: TokenResponse =
            get_signed(&[("method", "auth.getToken"), ("api_key", API_KEY)])?;
        Ok(response.token)
    })
    .await
}

pub fn auth_url(token: &str) -> String {
    format!("{AUTH_URL}?api_key={API_KEY}&token={token}")
}

pub async fn fetch_session(token: String) -> Result<Session, Error> {
    blocking(move || {
        let response: SessionResponse = get_signed(&[
            ("method", "auth.getSession"),
            ("api_key", API_KEY),
            ("token", token.as_str()),
        ])?;
        Ok(Session {
            username: response.session.name,
            key: response.session.key,
        })
    })
    .await
}

pub async fn update_now_playing(sk: String, song: StatusSong) -> Result<(), Error> {
    blocking(move || post_signed(track_params("track.updateNowPlaying", &sk, &song))).await
}

pub async fn scrobble(sk: String, song: StatusSong, started_at_unix: u64) -> Result<(), Error> {
    blocking(move || {
        let timestamp = started_at_unix.to_string();
        let mut params = track_params("track.scrobble", &sk, &song);
        params.push(("timestamp", &timestamp));
        post_signed(params)
    })
    .await
}
