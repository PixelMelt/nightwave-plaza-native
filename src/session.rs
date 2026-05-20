//! Persistent session storage.
//!
//! Saves/loads the auth token and cached user data to a JSON file in the
//! platform-standard config directory:
//!   Linux:   ~/.config/nightwave-plaza/session.json
//!   macOS:   ~/Library/Application Support/nightwave-plaza/session.json
//!   Windows: %APPDATA%/nightwave-plaza/session.json

use crate::api::User;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APP_DIR: &str = "nightwave-plaza";
const SESSION_FILE: &str = "session.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user: User,
}

fn session_path() -> Option<PathBuf> {
    let dir = dirs::config_dir()?.join(APP_DIR);
    Some(dir.join(SESSION_FILE))
}

/// Save a session (token + user) to disk.
pub fn save(token: &str, user: &User) {
    let Some(path) = session_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let session = Session {
        token: token.to_string(),
        user: user.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&session) {
        let _ = fs::write(&path, json);
    }
}

/// Load a previously saved session from disk.
pub fn load() -> Option<Session> {
    let path = session_path()?;
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Delete the saved session file (on logout).
pub fn clear() {
    if let Some(path) = session_path() {
        let _ = fs::remove_file(path);
    }
}
