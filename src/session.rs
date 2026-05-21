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

pub fn save(token: &str, user: &User) {
    let Some(path) = session_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let session = Session {
        token: token.to_string(),
        user: user.clone(),
    };
    if let Ok(bytes) = serde_json::to_vec(&session) {
        let _ = fs::write(&path, bytes);
    }
}

pub fn load() -> Option<Session> {
    let path = session_path()?;
    let file = fs::File::open(path).ok()?;
    serde_json::from_reader(file).ok()
}

pub fn clear() {
    if let Some(path) = session_path() {
        let _ = fs::remove_file(path);
    }
}
