use serde::{Deserialize, Serialize};

pub const API_KEY: &str = "e08418a04fb411affc437a8aab96cb89";
pub const API_SECRET: &str = "91a2c65df39ea28a7f305f7b35242e17";

const API_ROOT: &str = "https://ws.audioscrobbler.com/2.0/";
const AUTH_URL: &str = "https://www.last.fm/api/auth/";

pub fn is_configured() -> bool {
    !API_KEY.is_empty() && !API_SECRET.is_empty()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LastfmConfig {
    pub enabled: bool,
    pub session_key: Option<String>,
    pub username: Option<String>,
}

impl LastfmConfig {
    pub fn is_active(&self) -> bool {
        self.enabled && self.session_key.is_some() && is_configured()
    }
}

use crate::net::{agent, blocking};

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

fn body_of(result: Result<ureq::Response, ureq::Error>) -> Result<String, String> {
    match result {
        Ok(resp) => resp.into_string().map_err(|e| e.to_string()),
        Err(ureq::Error::Status(_, resp)) => resp.into_string().map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

fn parse<T: serde::de::DeserializeOwned>(
    result: Result<ureq::Response, ureq::Error>,
) -> Result<T, String> {
    let body = body_of(result)?;
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

pub async fn get_token() -> Result<String, String> {
    blocking(|| {
        let params = [("method", "auth.getToken"), ("api_key", API_KEY)];
        let sig = sign(&params);
        let parsed: TokenResp = parse(
            agent()
                .get(API_ROOT)
                .query("method", "auth.getToken")
                .query("api_key", API_KEY)
                .query("api_sig", &sig)
                .query("format", "json")
                .call(),
        )?;
        Ok(parsed.token)
    })
    .await
}

pub fn auth_url(token: &str) -> String {
    format!("{}?api_key={}&token={}", AUTH_URL, API_KEY, token)
}

pub async fn get_session(token: &str) -> Result<(String, String), String> {
    let token = token.to_string();
    blocking(move || {
        let params = [
            ("method", "auth.getSession"),
            ("api_key", API_KEY),
            ("token", token.as_str()),
        ];
        let sig = sign(&params);
        let parsed: SessionResp = parse(
            agent()
                .get(API_ROOT)
                .query("method", "auth.getSession")
                .query("api_key", API_KEY)
                .query("token", &token)
                .query("api_sig", &sig)
                .query("format", "json")
                .call(),
        )?;
        Ok((parsed.session.name, parsed.session.key))
    })
    .await
}

pub async fn update_now_playing(
    sk: &str,
    artist: &str,
    track: &str,
    album: &str,
) -> Result<(), String> {
    let (sk, artist, track, album) = (
        sk.to_string(),
        artist.to_string(),
        track.to_string(),
        album.to_string(),
    );
    blocking(move || {
        let mut params = vec![
            ("method", "track.updateNowPlaying"),
            ("api_key", API_KEY),
            ("sk", sk.as_str()),
            ("artist", artist.as_str()),
            ("track", track.as_str()),
        ];
        if !album.is_empty() {
            params.push(("album", album.as_str()));
        }
        post_signed(params)
    })
    .await
}

pub async fn scrobble(
    sk: &str,
    artist: &str,
    track: &str,
    album: &str,
    timestamp: u64,
) -> Result<(), String> {
    let (sk, artist, track, album) = (
        sk.to_string(),
        artist.to_string(),
        track.to_string(),
        album.to_string(),
    );
    blocking(move || {
        let ts = timestamp.to_string();
        let mut params = vec![
            ("method", "track.scrobble"),
            ("api_key", API_KEY),
            ("sk", sk.as_str()),
            ("artist", artist.as_str()),
            ("track", track.as_str()),
            ("timestamp", ts.as_str()),
        ];
        if !album.is_empty() {
            params.push(("album", album.as_str()));
        }
        post_signed(params)
    })
    .await
}

fn post_signed(params: Vec<(&str, &str)>) -> Result<(), String> {
    let sig = sign(&params);
    let mut form = params;
    form.push(("api_sig", sig.as_str()));
    form.push(("format", "json"));
    let _: serde_json::Value = parse(agent().post(API_ROOT).send_form(&form))?;
    Ok(())
}
