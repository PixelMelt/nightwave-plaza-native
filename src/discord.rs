use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver, Sender};

pub const CLIENT_ID: &str = "1511775400425160784";

pub fn is_configured() -> bool {
    !CLIENT_ID.is_empty()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub enabled: bool,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl DiscordConfig {
    pub fn is_active(&self) -> bool {
        self.enabled && is_configured()
    }
}

#[derive(Debug, Clone)]
pub struct Presence {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover_url: Option<String>,
    pub start_unix: i64,
    pub end_unix: Option<i64>,
}

enum Cmd {
    Set(Presence),
    Clear,
}

pub struct DiscordHandle {
    tx: Sender<Cmd>,
}

impl DiscordHandle {
    pub fn spawn() -> Option<Self> {
        if !is_configured() {
            return None;
        }
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || worker(rx));
        Some(Self { tx })
    }

    pub fn set(&self, presence: Presence) {
        let _ = self.tx.send(Cmd::Set(presence));
    }

    pub fn clear(&self) {
        let _ = self.tx.send(Cmd::Clear);
    }
}

fn worker(rx: Receiver<Cmd>) {
    let mut client = DiscordIpcClient::new(CLIENT_ID);
    let mut connected = false;
    let mut current: Option<Presence>;

    while let Ok(cmd) = rx.recv() {
        let mut cmd = cmd;
        while let Ok(next) = rx.try_recv() {
            cmd = next;
        }
        match cmd {
            Cmd::Set(p) => current = Some(p),
            Cmd::Clear => current = None,
        }

        if !connected {
            if client.connect().is_err() {
                continue;
            }
            connected = true;
        }

        if !apply(&mut client, current.as_ref()) {
            connected = false;
            if client.reconnect().is_ok() {
                connected = apply(&mut client, current.as_ref());
            }
        }
    }
}

fn apply(client: &mut DiscordIpcClient, presence: Option<&Presence>) -> bool {
    let Some(p) = presence else {
        return client.clear_activity().is_ok();
    };

    let title = p.title.trim();
    let artist = p.artist.trim();
    let album = p.album.trim();

    let mut assets = activity::Assets::new();
    let mut has_assets = false;
    if let Some(url) = p.cover_url.as_deref().filter(|u| !u.is_empty()) {
        assets = assets.large_image(url);
        assets = assets.large_text(if album.len() >= 2 {
            album
        } else {
            "Nightwave Plaza"
        });
        has_assets = true;
    }

    let mut ts = activity::Timestamps::new().start(p.start_unix * 1000);
    if let Some(end) = p.end_unix {
        ts = ts.end(end * 1000);
    }

    let mut act = activity::Activity::new()
        .activity_type(activity::ActivityType::Listening)
        .name("Nightwave Plaza Radio")
        .timestamps(ts);
    if title.len() >= 2 {
        act = act.details(title);
    }
    if artist.len() >= 2 {
        act = act.state(artist);
    }
    if has_assets {
        act = act.assets(assets);
    }

    client.set_activity(act).is_ok()
}
