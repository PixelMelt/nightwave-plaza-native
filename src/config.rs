use crate::api::User;
use crate::discord::DiscordConfig;
use crate::lastfm::LastfmConfig;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const APP_DIR: &str = "nightwave-plaza";
const CONFIG_FILE: &str = "config.json";
#[cfg(unix)]
const PRIVATE_DIR_MODE: u32 = 0o700;
#[cfg(unix)]
const PRIVATE_FILE_MODE: u32 = 0o600;

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
            let backup = path.with_extension(format!("json.{}.bad", Utc::now().timestamp()));
            eprintln!(
                "Failed to parse {}: {e}; moving it to {} and using defaults",
                path.display(),
                backup.display()
            );
            if let Err(e) = fs::rename(&path, &backup) {
                eprintln!("Failed to move {}: {e}", path.display());
            }
            Config::default()
        }
    }
}

pub fn save(cfg: &Config) {
    let Some(path) = config_path() else {
        eprintln!("No config directory found; settings not saved");
        return;
    };
    if let Err(e) = write_private(&path, cfg) {
        eprintln!("Failed to write {}: {e}", path.display());
    }
}

fn write_private(path: &Path, cfg: &Config) -> io::Result<()> {
    create_private_dir(path.parent().expect("config path has a parent"))?;
    let temporary = path.with_extension(format!("json.{}.tmp", std::process::id()));
    let mut file = create_private_file(&temporary)?;
    file.write_all(&serde_json::to_vec_pretty(cfg)?)?;
    file.sync_all()?;
    fs::rename(&temporary, path)
}

fn create_private_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir, fs::Permissions::from_mode(PRIVATE_DIR_MODE))?;
    }
    Ok(())
}

fn create_private_file(path: &Path) -> io::Result<File> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
        _ => {}
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(PRIVATE_FILE_MODE);
    }
    options.open(path)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn saved_config_is_private_and_replaces_the_old_file() {
        let dir = std::env::temp_dir().join(format!("nightwave-config-{}", std::process::id()));
        let path = dir.join(CONFIG_FILE);
        fs::create_dir_all(&dir).expect("create test dir");
        fs::write(&path, "stale").expect("seed stale config");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("loosen config");

        write_private(&path, &Config::default()).expect("write config");

        let file_mode = fs::metadata(&path).expect("config exists").permissions();
        let dir_mode = fs::metadata(&dir).expect("dir exists").permissions();
        let saved: Config =
            serde_json::from_slice(&fs::read(&path).expect("read config")).expect("config parses");
        fs::remove_dir_all(&dir).expect("remove test dir");

        assert_eq!(file_mode.mode() & 0o777, PRIVATE_FILE_MODE);
        assert_eq!(dir_mode.mode() & 0o777, PRIVATE_DIR_MODE);
        assert!(saved.session.is_none());
    }
}
