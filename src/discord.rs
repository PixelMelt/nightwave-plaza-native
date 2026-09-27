use crate::api::StatusSong;
use chrono::Utc;
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver, Sender};

const CLIENT_ID: &str = "1511775400425160784";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub enabled: bool,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

struct Presence {
    song: StatusSong,
    start_unix: i64,
}

pub struct Discord {
    tx: Sender<Option<Presence>>,
}

impl Discord {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || worker(&rx));
        Self { tx }
    }

    pub fn set(&self, song: &StatusSong) {
        let start_unix = Utc::now().timestamp() - song.position.max(0.0) as i64;
        let _ = self.tx.send(Some(Presence {
            song: song.clone(),
            start_unix,
        }));
    }

    pub fn clear(&self) {
        let _ = self.tx.send(None);
    }
}

fn worker(rx: &Receiver<Option<Presence>>) {
    let mut client = DiscordIpcClient::new(CLIENT_ID);
    let mut connected = false;
    while let Ok(mut presence) = rx.recv() {
        while let Ok(next) = rx.try_recv() {
            presence = next;
        }
        if !connected {
            if client.connect().is_err() {
                continue;
            }
            connected = true;
        }
        if !apply(&mut client, presence.as_ref()) {
            connected = client.reconnect().is_ok() && apply(&mut client, presence.as_ref());
        }
    }
}

fn apply(client: &mut DiscordIpcClient, presence: Option<&Presence>) -> bool {
    let Some(p) = presence else {
        return client.clear_activity().is_ok();
    };
    let song = &p.song;
    let title = song.title.trim();
    let artist = song.artist.trim();
    let album = song.album.trim();

    let mut ts = activity::Timestamps::new().start(p.start_unix * 1000);
    if song.length > 0.0 {
        ts = ts.end((p.start_unix + song.length as i64) * 1000);
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
    if let Some(url) = song.artwork_src.as_deref().filter(|u| !u.is_empty()) {
        let large_text = if album.len() >= 2 {
            album
        } else {
            "Nightwave Plaza"
        };
        act = act.assets(
            activity::Assets::new()
                .large_image(url)
                .large_text(large_text),
        );
    }

    client.set_activity(act).is_ok()
}
