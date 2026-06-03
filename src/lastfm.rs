use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APP_DIR: &str = "nightwave-plaza";
const CONFIG_FILE: &str = "lastfm.json";

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

fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join(APP_DIR).join(CONFIG_FILE))
}

pub fn load() -> LastfmConfig {
    let Some(path) = config_path() else {
        return LastfmConfig::default();
    };
    fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

pub fn save(cfg: &LastfmConfig) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(bytes) = serde_json::to_vec_pretty(cfg) {
        let _ = fs::write(&path, bytes);
    }
}

fn client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
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

async fn parse<T: serde::de::DeserializeOwned>(resp: reqwest::Response) -> Result<T, String> {
    let body = resp.text().await.map_err(|e| e.to_string())?;
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
    let params = [("method", "auth.getToken"), ("api_key", API_KEY)];
    let sig = sign(&params);
    let resp = client()
        .get(API_ROOT)
        .query(&params)
        .query(&[("api_sig", sig.as_str()), ("format", "json")])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let parsed: TokenResp = parse(resp).await?;
    Ok(parsed.token)
}

pub fn auth_url(token: &str) -> String {
    format!("{}?api_key={}&token={}", AUTH_URL, API_KEY, token)
}

pub async fn get_session(token: &str) -> Result<(String, String), String> {
    let params = [
        ("method", "auth.getSession"),
        ("api_key", API_KEY),
        ("token", token),
    ];
    let sig = sign(&params);
    let resp = client()
        .get(API_ROOT)
        .query(&params)
        .query(&[("api_sig", sig.as_str()), ("format", "json")])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let parsed: SessionResp = parse(resp).await?;
    Ok((parsed.session.name, parsed.session.key))
}

pub async fn update_now_playing(
    sk: &str,
    artist: &str,
    track: &str,
    album: &str,
) -> Result<(), String> {
    let mut params = vec![
        ("method", "track.updateNowPlaying"),
        ("api_key", API_KEY),
        ("sk", sk),
        ("artist", artist),
        ("track", track),
    ];
    if !album.is_empty() {
        params.push(("album", album));
    }
    post_signed(params).await
}

pub async fn scrobble(
    sk: &str,
    artist: &str,
    track: &str,
    album: &str,
    timestamp: u64,
) -> Result<(), String> {
    let ts = timestamp.to_string();
    let mut params = vec![
        ("method", "track.scrobble"),
        ("api_key", API_KEY),
        ("sk", sk),
        ("artist", artist),
        ("track", track),
        ("timestamp", ts.as_str()),
    ];
    if !album.is_empty() {
        params.push(("album", album));
    }
    post_signed(params).await
}

async fn post_signed(params: Vec<(&str, &str)>) -> Result<(), String> {
    let sig = sign(&params);
    let mut form = params;
    form.push(("api_sig", sig.as_str()));
    form.push(("format", "json"));
    let resp = client()
        .post(API_ROOT)
        .form(&form)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let _: serde_json::Value = parse(resp).await?;
    Ok(())
}
