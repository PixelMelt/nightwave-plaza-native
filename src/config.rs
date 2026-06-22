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
    let Ok(bytes) = fs::read(&path) else {
        return Config::default();
    };
    match serde_json::from_slice(&bytes) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to parse {}: {e}; using defaults", path.display());
            Config::default()
        }
    }
}

pub fn save(cfg: &Config) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let bytes = match serde_json::to_vec_pretty(cfg) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to serialize config: {e}");
            return;
        }
    };
    if let Err(e) = fs::write(&path, &bytes) {
        eprintln!("Failed to write {}: {e}", path.display());
    }
}
