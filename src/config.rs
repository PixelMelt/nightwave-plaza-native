use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::api::User;
use crate::discord::DiscordConfig;
use crate::lastfm::LastfmConfig;

const APP_DIR: &str = "nightwave-plaza";
const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub lastfm: LastfmConfig,
    pub discord: DiscordConfig,
    pub session: Option<Session>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user: User,
}

fn config_path() -> Option<PathBuf> {
    Some(crate::paths::config_dir()?.join(APP_DIR).join(CONFIG_FILE))
}

pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

pub fn save(cfg: &Config) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(bytes) = serde_json::to_vec_pretty(cfg) {
        let _ = fs::write(&path, bytes);
    }
}
