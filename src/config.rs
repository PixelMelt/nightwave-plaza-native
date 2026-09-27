use crate::api::User;
use crate::discord::DiscordConfig;
use crate::lastfm::LastfmConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .filter(|s| !s.is_empty())
            .map(|h| PathBuf::from(h).join("Library/Application Support"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    }
}

fn config_path() -> Option<PathBuf> {
    Some(config_dir()?.join(APP_DIR).join(CONFIG_FILE))
}

pub fn load() -> Config {
    let Some(path) = config_path() else {
        eprintln!("No config directory found; using defaults");
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
    let Some(path) = config_path() else {
        eprintln!("No config directory found; settings not saved");
        return;
    };
    let write = || -> std::io::Result<()> {
        fs::create_dir_all(path.parent().expect("config path has a parent"))?;
        fs::write(&path, serde_json::to_vec_pretty(cfg)?)
    };
    if let Err(e) = write() {
        eprintln!("Failed to write {}: {e}", path.display());
    }
}
