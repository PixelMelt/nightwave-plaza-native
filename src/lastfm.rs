use crate::api::StatusSong;
use crate::net::{agent, blocking, read_body};
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::time::Instant;

const API_KEY: &str = "e08418a04fb411affc437a8aab96cb89";
const API_SECRET: &str = "91a2c65df39ea28a7f305f7b35242e17";

const API_ROOT: &str = "https://ws.audioscrobbler.com/2.0/";
const AUTH_URL: &str = "https://www.last.fm/api/auth/";

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
}

pub struct Scrobble {
    pub song: StatusSong,
    start_unix: Option<u64>,
    played_secs: f64,
    playing_since: Option<Instant>,
}

impl Scrobble {
    pub fn new(song: StatusSong) -> Self {
        Self {
            song,
            start_unix: None,
            played_secs: 0.0,
            playing_since: None,
        }
    }

    pub fn set_playing(&mut self, playing: bool, now: Instant) {
        if playing {
            self.start_unix
                .get_or_insert_with(|| Utc::now().timestamp() as u64);
            self.playing_since.get_or_insert(now);
        } else if let Some(since) = self.playing_since.take() {
            self.played_secs += now.duration_since(since).as_secs_f64();
        }
    }

    pub fn due(&self, now: Instant) -> Option<u64> {
        let start = self.start_unix?;
        if !has_metadata(&self.song) {
            return None;
        }
        let length = self.song.length;
        if length > 0.0 && length < 30.0 {
            return None;
        }
        let threshold = if length > 0.0 {
            (length / 2.0).min(240.0)
        } else {
            30.0
        };
        let played = self.played_secs
            + self
                .playing_since
                .map_or(0.0, |since| now.duration_since(since).as_secs_f64());
        (played >= threshold).then_some(start)
    }
}

pub fn has_metadata(song: &StatusSong) -> bool {
    !song.artist.is_empty() && !song.title.is_empty()
}

fn sign(params: &[(&str, &str)]) -> String {
    let mut sorted: Vec<&(&str, &str)> = params.iter().filter(|(k, _)| *k != "format").collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    let mut buf = String::new();
    for (k, v) in sorted {
        buf.push_str(k);
        buf.push_str(v);
    }
    buf.push_str(API_SECRET);
    format!("{:x}", md5::compute(buf))
}

#[derive(Deserialize)]
struct LfmError {
    #[serde(default)]
    error: i32,
    #[serde(default)]
    message: String,
}

#[derive(Deserialize)]
struct TokenResp {
    token: String,
}

#[derive(Deserialize)]
struct SessionResp {
    session: SessionInner,
}

#[derive(Deserialize)]
struct SessionInner {
    name: String,
    key: String,
}

fn parse<T: DeserializeOwned>(result: Result<ureq::Response, ureq::Error>) -> Result<T, String> {
    let (_, body) = read_body(result)?;
    if let Ok(err) = serde_json::from_str::<LfmError>(&body) {
        if err.error != 0 {
            return Err(if err.message.is_empty() {
                format!("Last.fm error {}", err.error)
            } else {
                err.message
            });
        }
    }
    serde_json::from_str::<T>(&body).map_err(|e| e.to_string())
}

fn get_signed<T: DeserializeOwned>(params: &[(&str, &str)]) -> Result<T, String> {
    let sig = sign(params);
    let mut req = agent().get(API_ROOT);
    for (k, v) in params {
        req = req.query(k, v);
    }
    parse(req.query("api_sig", &sig).query("format", "json").call())
}

fn post_signed(params: Vec<(&str, &str)>) -> Result<(), String> {
    let sig = sign(&params);
    let mut form: Vec<(&str, &str)> = params;
    form.push(("api_sig", sig.as_str()));
    form.push(("format", "json"));
    let _: serde_json::Value = parse(agent().post(API_ROOT).send_form(&form))?;
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

pub async fn get_token() -> Result<String, String> {
    blocking(|| {
        let parsed: TokenResp = get_signed(&[("method", "auth.getToken"), ("api_key", API_KEY)])?;
        Ok(parsed.token)
    })
    .await
}

pub fn auth_url(token: &str) -> String {
    format!("{AUTH_URL}?api_key={API_KEY}&token={token}")
}

pub async fn get_session(token: String) -> Result<(String, String), String> {
    blocking(move || {
        let parsed: SessionResp = get_signed(&[
            ("method", "auth.getSession"),
            ("api_key", API_KEY),
            ("token", token.as_str()),
        ])?;
        Ok((parsed.session.name, parsed.session.key))
    })
    .await
}

pub async fn update_now_playing(sk: String, song: StatusSong) -> Result<(), String> {
    blocking(move || post_signed(track_params("track.updateNowPlaying", &sk, &song))).await
}

pub async fn scrobble(sk: String, song: StatusSong, timestamp: u64) -> Result<(), String> {
    blocking(move || {
        let ts = timestamp.to_string();
        let mut params = track_params("track.scrobble", &sk, &song);
        params.push(("timestamp", ts.as_str()));
        post_signed(params)
    })
    .await
}
